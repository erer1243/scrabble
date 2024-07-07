import { GameT, InvalidMoveT, MoveT } from "./game-types"

export const serverAddr: string = `ws://${document.location.hostname}:2222/`

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
| "ExchangeTiles"
