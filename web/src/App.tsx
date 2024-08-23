import { useEffect, useState } from 'react'
import useWebSocket, { ReadyState } from 'react-use-websocket'
import { ClientMessageT, ServerMessageT, TableT, serverAddr } from './client'
import { GameView } from './GameView'
import { MoveT } from './game-types'
import { SetupView } from './SetupView'
import { DebugInfo } from './DebugInfo'
import './App.scss'

// When serving with vite dev server, this is true https://vitejs.dev/guide/env-and-mode
const debugMode = import.meta.env.DEV

const getStoredName = () => localStorage["name"]
const setStoredName = (name: string) => localStorage["name"] = name
const delStoredName = () => localStorage.removeItem("name")

const App = () => {
  const { sendJsonMessage, lastJsonMessage, readyState } = useWebSocket(serverAddr, {
    reconnectAttempts: 20,
    reconnectInterval: attemptNumber => attemptNumber < 10 ? 1000 : 5000, // ms
    shouldReconnect: () => true,
  })
  const sendMessage = (m: ClientMessageT) => sendJsonMessage(m)

  const [table, setTable] = useState<TableT | undefined>(undefined)
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
  }, [readyState])

  // Handle the latest message from the server
  useEffect(() => {
    if (lastJsonMessage) {
      const msg = lastJsonMessage as ServerMessageT
      if ("Table" in msg) {
        setTable(msg.Table)
      } else if ("InvalidMove" in msg) {
        alert(msg.InvalidMove.explanation)
      } else {
        alert("Unhandled ServerMessage (see console)")
        console.error("Unhandled ServerMessage", msg);
      }
    }
  }, [lastJsonMessage])

  // When the user enters their name, send that name to the server and add it to local storage
  useEffect(() => {
    if (name !== undefined) {
      setStoredName(name)
      sendMessage({ "JoinWithName": name })
    }
  }, [name])

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
    case "Setup":
      const startGame = () => {
        if (confirm("Start game for everyone? (only do this once everyone has joined)"))
          sendMessage("StartGame")
      }
      elems.push(<SetupView key="setup" game={table.game} joinGame={setName} name={name} startGame={startGame} />)
      break;
    case "Running":
      const playMove = (move: MoveT) => sendMessage({ "PlayMove": move })
      const exchangeTiles = () => sendMessage("ExchangeTiles")
      elems.push(<GameView key="game" game={table.game} name={name} playMove={playMove} exchangeTiles={exchangeTiles} />)
      break;
    case undefined:
      elems.push(<h1 key="notconnected" style={{ color: 'white' }}>Not connected</h1>)
      break;
  }

  if (debugMode) {
    const statuses = {
      [ReadyState.CONNECTING]: 'connecting',
      [ReadyState.OPEN]: 'open',
      [ReadyState.CLOSING]: 'closing',
      [ReadyState.CLOSED]: 'closed',
      [ReadyState.UNINSTANTIATED]: 'uninstantiated',
    }

    const debugData = {
      "Socket is": statuses[readyState],
      name,
      storedName: getStoredName(),
      table,
      lastJsonMessage,
    }
    elems.push(<br key="debugbreak" />)
    elems.push(<DebugInfo key="debuginfo" data={debugData} />)
  }

  return elems
}

export default App
