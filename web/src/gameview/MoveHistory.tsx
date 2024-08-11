import { GameT, TurnT, tileValues } from "../game-types"
import "./MoveHistory.scss"

export type MoveHistoryProps = {
  game: GameT
}

const turnDescription = (turn: TurnT): string => {
  if (turn === "TilesExchanged") {
    return "exchanged their tiles"
  } if ("PlayedMove" in turn) {
    const words = turn.PlayedMove.word_values.map(([word, _val]) => word.toUpperCase()).join(", ")
    const value = turn.PlayedMove.word_values.reduce((subscore, [_word, val]) => subscore + val, 0)
    return `played ${words} for ${value} points`
  } else /* ("GameEnd" in turn) */ {
    if ("RemainingTiles" in turn.GameEnd) {
      const lostPoints = turn.GameEnd.RemainingTiles.reduce((sum, tile) => sum + tileValues[tile], 0)
      const remainingTiles = turn.GameEnd.RemainingTiles.join(", ");
      return `lost ${lostPoints} points by finishing with [${remainingTiles}]`
    } else /* ("PlayedLastMove" in turn.GameEnd) */ {
      return `gained ${turn.GameEnd.PlayedLastMove} points for playing the final move`
    }
  }
}

const turnDescriptions = (game: GameT): Array<string> => {
  const descs = [];
  let turnIdx = 0;
  outer: for (;;) {
    for (const player of game.players) {
      if (turnIdx < player.turns.length) {
        const desc = turnDescription(player.turns[turnIdx])
        descs.push(`${player.name} ${desc}`)
      } else {
        break outer
      }
    }

    turnIdx++
  }
  descs.reverse()
  return descs
}

export const MoveHistory = ({ game }: MoveHistoryProps) => {
  const listItems = turnDescriptions(game).map((s, i) => (
    <li key={i} className="item">{s}</li>
  ));
  return (
    <div className="move-history">
      <div className="title-div">
        <h3 className="title">Move History</h3>
      </div>
      <ul className="list">
        {listItems}
      </ul>
    </div>
  )
}

