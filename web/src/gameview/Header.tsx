import { GameT, PlayerT, tileValues } from "../game-types"
import "./Header.scss"

export type GameViewHeaderProps = {
  game: GameT
  name: string | undefined
}

const scoreOfPlayer = (p: PlayerT): number =>
  p.turns.reduce((score, turn): number => {
    if (turn === "TilesExchanged") {
      return score
    } if ("PlayedMove" in turn) {
      const value = turn.PlayedMove.word_values.reduce((subscore, word) => subscore + word[1], 0)
      return score + value
    } else /* ("GameEnd" in turn) */ {
      if ("RemainingTiles" in turn.GameEnd) {
        return score - turn.GameEnd.RemainingTiles.reduce((sum, tile) => sum + tileValues[tile], 0)
      } else /* ("PlayedLastMove" in turn.GameEnd) */ {
        return score + turn.GameEnd.PlayedLastMove
      }
    }
  }, 0)

export const Header = ({ game, name }: GameViewHeaderProps) => {
  const curPlayer = game.players[game.whose_turn]
  const scores = game.players.map((p, i) =>
    <div className="row" key={i}>
      <p className="name">{p.name}</p>
      <p className="score">{scoreOfPlayer(p)}</p>
    </div>
  )

  let whoseTurnMessage;
  if (game.finished) {
    whoseTurnMessage = "Game over"
  } else {
    whoseTurnMessage = `It's ${curPlayer.name == name ? "your" : `${curPlayer.name}'s`} turn`
  }

  return (
    <div className="game-view-header">
      <div className="scores">
        <div className="row"><p className="name">Player</p><p className="score">Score</p></div>
        {scores}
      </div>
      <div className="whose-turn">
        <h3>{whoseTurnMessage}</h3>
      </div>
    </div>
  )
}
