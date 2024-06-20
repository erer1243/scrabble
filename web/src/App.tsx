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

const debugMode = import.meta.env.DEV

const getStoredName = () => localStorage["name"]
const setStoredName = (name: string) => localStorage["name"] = name
const delStoredName = () => localStorage.removeItem("name")

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
        alert(msg.InvalidMove.explanation)
      } else {
        alert("Unhandled ServerMessage (see console)")
        console.error("Unhandled ServerMessage", msg);
      }
    }
  }, [lastJsonMessage])

  useEffect(() => {
    if (name !== undefined) {
      setStoredName(name)
      sendMessage({ "JoinWithName": name })
    }
  }, [name, sendMessage])

  useEffect(() => {
    const storedName = getStoredName()
    if (name === undefined && table !== undefined && storedName !== undefined) {
      if (table.game.players.some(p => p.name == storedName))
        // We have a stored name and it's in the game, so use it
        setName(storedName)
      else
        // We have a stored name but it's not in the game, so it's outdated and needs to be removed
        delStoredName()
    }
  }, [table, name])

  const elems = [];
  switch (table?.state) {
    case "Setup": {
      const startGame = () => sendMessage("StartGame")
      elems.push(<SetupView key="setup" game={table.game} joinGame={setName} name={name} startGame={startGame} />)
      break;
    }

    case "Running": {
      const playMove = (move: MoveT) => sendMessage({ "PlayMove": move })
      elems.push(<GameView key="game" game={table.game} name={name} playMove={playMove} />)
      break;
    }

    case undefined: {
      elems.push(<h1 key="notconnected" style={{ color: 'white' }}>Not connected</h1>)
      break;
    }
  }

  if (debugMode) {
    const debugData = {
      "Socket is": statuses[readyState],
      name,
      storedName: getStoredName(),
      table,
      lastJsonMessage,
      invalidMove,
    }
    elems.push(<br key="debugbreak" />)
    elems.push(<DebugInfo key="debuginfo" data={debugData} />)
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
