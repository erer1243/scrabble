import "./Tile.css";
import background from "./assets/tile/background.png";
import { TileT, tileValues } from "./game";

export type TileAttributes = {
  tile: TileT,
  pointValue?: number,
  selectable?: boolean,
  selected?: boolean,
};

export const Tile = ({ tile: letter, pointValue, selectable, selected }: TileAttributes) => {
  letter = letter.toUpperCase() as TileT;
  pointValue = pointValue ?? tileValues[letter];
  let className = `tile ${selectable ? "tile-selectable" : ""} ${selected ? "tile-selected" : ""}`

  return (
    <div className={className}>
      <p className="tile-letter">{letter}</p>
      <p className="tile-point-value">{pointValue}</p>
      <img className="tile-background" draggable="false" src={background} />
    </div>
  )
}
