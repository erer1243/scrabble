import { useEffect, useState } from 'react'
import { ServerMessageT, TableT, serverAddr } from './client'
import useWebSocket, { ReadyState } from 'react-use-websocket'

import './App.scss'
import { GameView } from './GameView'

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

const showDebugInfoByDefault = true;

const App = () => {
  const { sendJsonMessage, lastJsonMessage, readyState } = useWebSocket(serverAddr, sockOptions);
  const [table, setTable] = useState<TableT | undefined>(undefined);

  useEffect(() => {
    if (lastJsonMessage) {
      const serverMessage = lastJsonMessage as ServerMessageT;
      setTable(serverMessage.Update.table);
    }
  }, [lastJsonMessage])

  useEffect(() => {
    if (readyState !== ReadyState.OPEN)
      setTable(undefined);
  }, [readyState])

  let gameView = undefined;
  if (table) {
    gameView = (
      <GameView game={table.game} playerIndex={0} />
    )
  }

  return (
    <>
      {gameView}
      <DebugInfo data={[`Socket is ${statuses[readyState]}`, lastJsonMessage]} />
    </>
  )
}

const DebugInfo = ({ data }: { data: any[] }) => {
  const [show, setShow] = useState(showDebugInfoByDefault);
  const white = { color: "white" }

  let list;
  if (show) {
    const listItems = data.map((val, i) => <li key={i} style={white}><pre style={white}>{JSON.stringify(val, null, 2)}</pre></li>);
    list = <ul style={{ border: "1px solid white", borderRadius: "3px" }}>{listItems}</ul>
  }

  return (
    <>
      <button onClick={() => setShow(!show)}>toggle debug info</button>
      {list}
    </>
  )
}

export default App
