import "./Tile.scss";
import background from "./assets/tile/background.png";
import { BoardTileT, TileT, tileValues } from "./game-types";

export type TileProps = {
  tile: TileT | BoardTileT
};

export const Tile = ({ tile }: TileProps) => {

  let tileLetter, pointValue, tileLetterClass = "tile-letter";

  if (typeof tile === 'string') {
    if (tile === 'Blank') {
      tileLetter = ""
    } else {
      tileLetter = tile
    }
    pointValue = tileValues[tile]
  } else if ('Blank' in tile) {
    tileLetter = tile['Blank']
    pointValue = tileValues['Blank']
    tileLetterClass += " tile-filled-blank"
  }

  return (
    <div className="tile">
      <p className={tileLetterClass}>{tileLetter}</p>
      <p className="tile-point-value">{pointValue}</p>
      <img className="tile-background" draggable="false" src={background} />
    </div>
  )
}
