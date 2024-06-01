import { useState } from "react"
import { Board } from "./Board"
import { TileBar } from "./TileBar"
import { GameT } from "./game"

import "./GameView.scss"

export type GameViewProps = {
  game: GameT,
  playerIndex: number,
}

export const GameView = ({ game, playerIndex }: GameViewProps) => {
  const [selectedTile, setSelectedTile] = useState<number | undefined>(undefined);
  const tiles = game.players[playerIndex].tiles;

  return (
    <div className="game-view">
      <div className="game-view-tile-bar">
        <h2 className="game-view-tile-bar-label">Your Tiles:</h2>
        <TileBar tiles={tiles} onClickTile={setSelectedTile} selectedTile={selectedTile} />
      </div>
      <Board board={game.board} onClickSquare={console.log} />
    </div>
  )
}
