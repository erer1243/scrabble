import { useState } from "react"
import { Board } from "./Board"
import { TileBar } from "./TileBar"
import { GameT } from "./game"

export type GameViewProps = {
  game: GameT,
  playerIndex: number,
}

export const GameView = ({ game, playerIndex }: GameViewProps) => {
  const [selectedTile, setSelectedTile] = useState<number | undefined>(undefined);
  const tiles = game.players[playerIndex].tiles;

  return (
    <div>
      <TileBar tiles={tiles} onClickTile={setSelectedTile} selectedTile={selectedTile} />
      <Board board={game.board} onClickSquare={console.log} />
    </div>
  )
}
