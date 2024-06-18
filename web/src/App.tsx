import './App.scss'
import { useCallback, useEffect, useState } from 'react'
import { ClientMessageT, ServerMessageT, TableT, serverAddr } from './client'
import useWebSocket, { ReadyState } from 'react-use-websocket'
import { GameView } from './GameView'
import { InvalidMoveT, MoveT } from './game-types'
import { SetupView } from './SetupView'

const statuses = {
  [ReadyState.CONNECTING]: 'connecting',
  [ReadyState.OPEN]: 'open',
  [ReadyState.CLOSING]: 'closing',
  [ReadyState.CLOSED]: 'closed',
  [ReadyState.UNINSTANTIATED]: 'uninstantiated',
}

const sockOptions = {
  reconnectAttempts: 20,
  reconnectInterval: 5000, // ms
  shouldReconnect: () => true,
}

const debugMode = true

const randomName = (): string => {
  const randomElem = (s: string): string => s[Math.trunc(Math.random() * s.length)]
  const consonants = "bcdfghjklmnpqrstvwxyz"
  const vowels = "aeiou"

  const lengthSeed = Math.random()
  let name = randomElem(consonants).toUpperCase();
  for (let i = 0; lengthSeed < (1 / 1.7**i) && i <= 5; i++)
    name += randomElem(vowels) + randomElem(consonants)
  
  return name
}

const getCookieName = () => localStorage["name"]
const setCookieName = (name: string) => localStorage["name"] = name
const unsetCookieName = () => localStorage.removeItem("name")

const App = () => {
  const { sendJsonMessage, lastJsonMessage, readyState } = useWebSocket(serverAddr, sockOptions)
  const sendMessage = useCallback((m: ClientMessageT) => sendJsonMessage(m), [sendJsonMessage])

  const [table, setTable] = useState<TableT | undefined>(undefined)
  const [invalidMove, setInvalidMove] = useState<InvalidMoveT | undefined>(undefined)
  const [name, setName] = useState<string | undefined>(undefined)

  // If readyState changes to OPEN, send a request for an update.
  // If readyState is anything else, clear the table.
  useEffect(() => {
    if (readyState === ReadyState.OPEN) {
      sendMessage("UpdateMe")
    } else {
      setTable(undefined)
      setName(undefined)
    }
  }, [readyState, sendMessage])

  // Handle the latest message from the server
  useEffect(() => {
    if (lastJsonMessage) {
      const msg = lastJsonMessage as ServerMessageT
      if ("Table" in msg) {
        setTable(msg.Table)
      } else if ("InvalidMove" in msg) {
        setInvalidMove(msg.InvalidMove)
      } else {
        alert("Unhandled ServerMessage (see console)")
        console.error("Unhandled ServerMessage", msg);
      }
    }
  }, [lastJsonMessage])

  useEffect(() => {
    if (name !== undefined) {
      setCookieName(name)
      sendMessage({ "JoinWithName": name })
    }
  }, [name, sendMessage])

  useEffect(() => {
    const cookieName = getCookieName()
    if (name === undefined && table !== undefined && cookieName !== undefined) {
      if (table.game.players.some(p => p.name == cookieName))
        // We have a cookie name and it's in the game, so use it
        setName(cookieName)
      else
        // We have a cookie name but it's not in the game, so it's outdated and needs to be removed
        unsetCookieName()
    }
  }, [table, name])

  useEffect(() => {
    if (debugMode && table?.state === "Setup" && name === undefined && getCookieName() === undefined)
      setName(randomName())
  }, [table?.state, name])

  const elems = [];
  switch (table?.state) {
    case "Setup": {
      const startGame = () => sendMessage("StartGame")
      elems.push(<SetupView game={table.game} joinGame={setName} name={name} startGame={startGame} />)
      break;
    }

    case "Running": {
      const playMove = (move: MoveT) => sendMessage({ "PlayMove": move })
      elems.push(<GameView game={table.game} name={name} playMove={playMove} />)
      break;
    }

    case "Review": {
      elems.push(<h1 style={{ color: 'white '}}>Review screen now!</h1>)
      break;
    }

    case undefined: {
      elems.push(<h1 style={{ color: 'white' }}>Not connected</h1>)
      break;
    }
  }

  if (debugMode) {
    const debugData = {
      "Socket is": statuses[readyState],
      name,
      cookieName: getCookieName(),
      table,
      lastJsonMessage,
      invalidMove,
    }
    elems.push(<br />)
    elems.push(<DebugInfo data={debugData} />)
  }
  return elems
}

const DebugInfo = ({ data }: { data: Record<string, unknown> }) => {
  const [show, setShow] = useState(false)
  const white = { color: "white" }
  const blacklist = ["board", "tile_bag", "tiles"]

  let list
  if (show) {
    const stringify = (x: unknown) => {
      return JSON.stringify(x, (k, v) => blacklist.includes(k) ? "(redacted)" : v, 1)
    }

    const listItems = Object.entries(data).map(([label, val], i) =>
      <li key={i} style={white}>
        <pre style={white}>
          {label}: {stringify(val)}
        </pre>
      </li>)
    list = <ul style={{ border: "1px solid white", borderRadius: "3px" }}>{listItems}</ul>
  }

  return (
    <>
      <button onClick={() => setShow(!show)}>debug info</button>
      {list}
    </>
  )
}

export default App
