import { Tile } from "./Tile"
import { TileT } from "../game-types"
import "./TileBar.scss"

export type TileBarProps = {
  tiles: Array<TileT>,
  onClickTile: (i: number) => void,
  selectedTile: number | undefined,
}

// XXX: Key being the index of the letter is apparently buggy and wrong?
// https://react.dev/learn/rendering-lists#why-does-react-need-keys
export const TileBar = ({ tiles, onClickTile, selectedTile }: TileBarProps) => {
  const tileElems = tiles.map((l, i) => {
    const selectedClass = (selectedTile === i) ? "tile-bar-tile-selected" : "tile-bar-tile-unselected"
    return (
      <div className={`tile-bar-tile ${selectedClass}`} onClick={() => onClickTile(i)} key={i}>
        <Tile tile={l} />
      </div>
    )
  })
  return (
    <div className="tile-bar" >
      {tileElems}
    </div>
  )
}
