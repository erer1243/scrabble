import "./Tile.scss";
import background from "./assets/tile/background.png";
import { TileT, tileValues } from "./game-types";

export type TileProps = {
  tile: TileT,
  pointValue?: number,
};

export const Tile = ({ tile, pointValue }: TileProps) => {
  pointValue ??= tileValues[tile];
  const tileLetter = (tile == "Blank") ? "" : tile as string;
  
  return (
    <div className="tile">
      <p className="tile-letter">{tileLetter}</p>
      <p className="tile-point-value">{pointValue}</p>
      <img className="tile-background" draggable="false" src={background} />
    </div>
  )
}
