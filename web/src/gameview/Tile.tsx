import "./Tile.scss";
import background from "./tile.png";
import { BoardTileT, TileT, tileValues } from "../game-types";

export type TileProps = {
  tile: TileT | BoardTileT
};

const nudges: Record<string, string> = {
  "D": " nudge-left-2",
  "G": " nudge-left-2",
  "H": " nudge-left-2",
  "I": " nudge-right-3",
  "J": " nudge-right-4 nudge-up-2",
  "M": " nudge-left-4",
  "N": " nudge-left-2",
  "O": " nudge-left-2",
  "Q": " nudge-left-3 nudge-up-2",
  "U": " nudge-left-1",
  "W": " nudge-left-4",
  "Z": " nudge-up-1",
}

export const Tile = ({ tile }: TileProps) => {
  let tileLetter: string
  let pointValue: number
  let tileLetterClass: string = "letter"

  if (typeof tile === 'string') {
    // TileT
    tileLetter = (tile === 'Blank') ? "" : tile
    pointValue = tileValues[tile]
  } else if ('Blank' in tile) {
    // BoardTileT
    tileLetter = tile['Blank']
    pointValue = tileValues['Blank']
    tileLetterClass += " filled-blank"
  } else {
    console.error('impossible')
    return
  }

  tileLetterClass += nudges[tileLetter] ?? ""

  return (
    <div className="tile">
      <p className={tileLetterClass}>{tileLetter}</p>
      <p className="point-value">{pointValue}</p>
      <img className="background" draggable="false" src={background} />
    </div>
  )
}
