import { GameT } from "./game"

export type TableT = {
  game: GameT,
}

export type ServerMessageT = {
  Update: { table: TableT }
}

export type ClientMessageT = {
  JoinTable: { id: string }
}

export const serverAddr: string = "ws://192.168.1.4:2222"

