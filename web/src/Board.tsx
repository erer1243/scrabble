import { Tile } from "./Tile";
import { BoardT, modifiers, newBoard } from "./game"
import "./Tile.css"

export type BoardAttributes = {
  board: BoardT,
}

export const Board = ({ board }: BoardAttributes) => {
  let tileElemBoard: BoardT<React.JSX.Element | undefined> = newBoard();

  for (let x = 0; x < 15; x++) {
    for (let y = 0; y < 15; y++) {
      let tile = board[x][y];
      let elem;
      if (tile) {
        elem = <Tile tile={tile} />
      } else if (modifiers[x][y]) {
        elem = <div className="tile" style={{"backgroundColor": "red"}}></div>
      } else {
        elem = <div className="tile" style={{"backgroundColor": "green"}}></div>
      }

      elem.props["key"] =`${x},${y}` 

      tileElemBoard[x][y] = elem;
    }
  }

  return tileElemBoard[0]
}
