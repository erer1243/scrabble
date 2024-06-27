import { GameT, PlayerT } from "../game-types"
import "./Header.scss"

export type GameViewHeaderProps = {
  game: GameT
  name: string | undefined
}

const scoreOfPlayer = (p: PlayerT): number =>
  p.turns.reduce((score, turn): number => {
    if (turn === "TilesExchanged") {
      return 0;
    } else {
      const value = turn.PlayedMove.word_values.reduce((subscore, word) => subscore + word[1], 0);
      return score + value;
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

  return (
    <div className="game-view-header">
      <div className="scores">
        <div className="row"><p className="name">Player</p><p className="score">Score</p></div>
        {scores}
      </div>
      <div className="whose-turn">
        <h3>It's {curPlayer.name == name ? "your" : `${curPlayer.name}'s`} turn</h3>
      </div>
    </div>
  )
}
