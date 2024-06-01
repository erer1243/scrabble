import { Tile } from "./Tile"
import { TileT } from "./game"
import "./TileBar.scss"

export type TileBarProps = {
  tiles: Array<TileT>,
  onClickTile: (i: number) => void,
  selectedTile: number | undefined,
}

export const TileBar = ({ tiles, onClickTile, selectedTile }: TileBarProps) => {
  // XXX: Key being the index of the letter is apparently buggy and wrong?
  // https://react.dev/learn/rendering-lists#why-does-react-need-keys
  const tileElems = tiles.map((l, i) => {
    const className = (selectedTile == i) ? "tile-bar-tile tile-bar-tile-selected" : "tile-bar-tile";
    return (
      <div className={className} onClick={() => onClickTile(i)} key={i}>
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
