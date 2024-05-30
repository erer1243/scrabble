import { Tile } from "./Tile"
import { TileT } from "./game"
import "./TileBar.css"

export type TileBarAttributes = {
  letters: Array<TileT>,
}

export const TileBar = ({ letters }: TileBarAttributes) => {
  const tileElems = letters.map(l => (
    <div className="tile-bar-tile">
      <Tile tile={l} selectable />
    </div>
  ))
  return (
    <div className="tile-bar" >
      {tileElems}
    </div>
  )
}
