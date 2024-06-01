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

const App = () => {
  const { sendJsonMessage, lastJsonMessage, readyState } = useWebSocket(serverAddr, {
    reconnectAttempts: 20,
    reconnectInterval: 5000, // ms
    shouldReconnect: () => true,
  });

  const [table, setTable] = useState<TableT | undefined>(undefined);
  void ([table, setTable, sendJsonMessage]);

  useEffect(() => {
    if (lastJsonMessage) {
      // console.log(lastJsonMessage)
      const serverMessage = lastJsonMessage as ServerMessageT;
      setTable(serverMessage.Update.table);
    }
  }, [lastJsonMessage])

  let gameView = undefined;
  if (table) {
    gameView = (
      <GameView game={table.game} playerIndex={0} />
    )
  }

  return (
    <>
      {gameView}
      <h1 style={{ "color": "white" }}>Socket is {statuses[readyState]}</h1>
      <pre style={{ "color": "white" }}>{JSON.stringify(lastJsonMessage, null, 1)}</pre>
    </>
  )
}

export default App
