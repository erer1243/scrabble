import { GameT, InvalidMoveT, MoveT } from "./game-types"

export const serverAddr: string = import.meta.env.DEV ? "ws://127.0.0.1:2222/" : "ws://oliver.radiobike.com:2222/"

export type TableT = {
  game: GameT
  state: GameStateT
}

export type GameStateT = "Setup" | "Running"

export type ServerMessageT = 
| { Table: TableT }
| { InvalidMove: InvalidMoveT }

export type ClientMessageT = 
| "UpdateMe"
| "StartGame"
| { JoinWithName: string }
| { PlayMove: MoveT }
