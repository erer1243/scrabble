import { useEffect, useState } from "react"
import { Board } from "./Board"
import { TileBar } from "./TileBar"
import { BoardT, DirectionT, GameT, MoveT, PointT, TileT } from "./game"

import "./GameView.scss"

export type GameViewProps = {
  game: GameT,
  playerIndex: number,
}

// class GameViewState {
//   availableTiles: Array<TileT>
//   selectedTile: number | undefined
//   boardWithMovesApplied: BoardT

//   rerender: () => void

//   constructor(game: GameT, playerIndex: number) {
//     this.availableTiles = structuredClone(game.players[playerIndex].tiles)
//     this.selectedTile = undefined
//     this.boardWithMovesApplied = structuredClone(game.board)
//   }
// }

// const makeSetters = (obj: any, setObj: any): Record<any, (val: any) => void> =>
//   Object.fromEntries(Object.entries(obj).map(([key]) => [key, (val: any) => setObj({ ...obj, [key]: val })]))

// type InsertedTiles = Record<string, [PointT, TileT]>

type InsertedTiles = Array<[PointT, TileT]>

const arrAppend = (arr: Array<any>, val: any): Array<any> => [...arr, val]
const arrRemove = (arr: Array<any>, i: number): Array<any> => arr.filter((_val, idx) => idx !== i)
const applyInsertedTiles = (board: BoardT, insertedTiles: InsertedTiles): BoardT => {
  const newBoard = structuredClone(board)
  for (const [[x, y], tile] of insertedTiles) {
    newBoard[x][y] = tile
  }
  return newBoard
}

export const GameView = ({ game, playerIndex }: GameViewProps) => {
  const [availableTiles, setAvailableTiles] = useState<Array<TileT>>(structuredClone(game.players[playerIndex].tiles))
  const [selectedTile, setSelectedTile] = useState<number | undefined>(undefined)
  const [insertedTiles, setInsertedTiles] = useState<InsertedTiles>([])

  const [resetMarker, setResetMarker] = useState(false)
  const resetState = () => setResetMarker(!resetMarker) 

  const isMyTurn = game.whose_turn == playerIndex
  
  useEffect(() => {
    setAvailableTiles(structuredClone(game.players[playerIndex].tiles))
    setSelectedTile(undefined)
    setInsertedTiles([])
  }, [isMyTurn, resetMarker]);

  return (
    <div className="game-view">
      <div className="game-view-tile-bar">
        <h2 className="game-view-tile-bar-label">Your Tiles:</h2>
        <TileBar tiles={availableTiles} onClickTile={setSelectedTile} selectedTile={selectedTile} />
      </div>
      <Board board={applyInsertedTiles(game.board, insertedTiles)} onClickSquare={(x, y, isFilled) => {
        if (selectedTile !== undefined && !isFilled) {
          setInsertedTiles(arrAppend(insertedTiles, [[x, y], availableTiles[selectedTile]]))
          setAvailableTiles(arrRemove(availableTiles, selectedTile))
          setSelectedTile(undefined)
        }
      }} />
      <button onClick={resetState}>Reset GameView state</button>
    </div>
  )
}

