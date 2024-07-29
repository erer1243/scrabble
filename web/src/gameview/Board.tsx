import { Tile } from "./Tile"
import { BoardT, modifiers } from "../game-types"
import "./Board.scss"

export type BoardProps = {
  board: BoardT,
  onClickSquare: (x: number, y: number, isFilled: boolean) => void,
}

export const Board = ({ board, onClickSquare }: BoardProps) => {
  const rows: Array<React.JSX.Element> = [];
  for (let x = 0; x < 15; x++) {
    const rowSquares: Array<React.JSX.Element> = [];
    for (let y = 0; y < 15; y++) {
      const tile = board[x][y]
      const modifier = modifiers[x][y]

      let elem;
      let isFilled = false;
      if (tile) {
        isFilled = true;
        elem = (
          <Tile tile={tile} />
        )
      } else if (modifier) {
        const [modifierText, modifierClassName] = {
          "DoubleLetter": ["Double Letter Score", "double-letter"],
          "TripleLetter": ["Triple Letter Score", "triple-letter"],
          "DoubleWord": ["Double Word Score", "double-word"],
          "TripleWord": ["Triple Word Score", "triple-word"],
        }[modifier]
        elem = (
          <div className={`empty modifier ${modifierClassName}`}>
            <p className="modifier-text">{modifierText}</p>
          </div>
        )
      } else {
        elem = <div className="empty-square"></div>
      }

      rowSquares[y] = (
        <div className="square" onClick={() => onClickSquare(x, y, isFilled)} key={`board-square-${y}`}>
          {elem}
        </div>
      )
    }

    rows[x] = (
      <div className="row" key={`row-${x}`}>
        {rowSquares}
      </div>
    )
  }

  return (
    <div className="board">
      {rows}
    </div>
  )
}
