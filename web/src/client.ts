import { BoardT, GameT } from "./game"

export const serverAddr: string = "ws://127.0.0.1:2222/"

export type TableT = {
  game: GameT,
  players: Array<string>,
}

export type ServerMessageT = 
  { Update: { board: BoardT } }
| { TableInfo: { table: TableT } }

export type ClientMessageT = 
  { JoinTable: { id: string } }
| { GetTableInfo: { table: string } }

