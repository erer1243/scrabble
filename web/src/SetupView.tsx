import { GameT } from "./game-types"
import "./SetupView.scss"

export type SetupViewProps = {
  game: GameT
  name: string | undefined
  joinGame: (name: string) => void
  startGame: () => void
}

const randomName = (): string => {
  const randomElem = (s: string): string => s[Math.trunc(Math.random() * s.length)]
  const consonants = "bcdfghjklmnpqrstvwxyz"
  const vowels = "aeiou"

  const lengthSeed = Math.random()
  let name = randomElem(consonants).toUpperCase();
  for (let i = 0; lengthSeed < (1 / 1.7 ** i) && i <= 5; i++) {
    const numVowel = Math.random() < 0.1 ? 2 : 1;
    name += randomElem(vowels).repeat(numVowel) + randomElem(consonants)
  }

  return name
}

export const SetupView = ({ game, name, joinGame, startGame }: SetupViewProps) => {
  let joinGameArea
  if (name === undefined) {
    const onClickJoin = () => {
      const input = (document.getElementById("name-input")! as HTMLInputElement).value.trim()
      if (input)
        joinGame(input)
      else
        alert("Enter a name")
    }
    const onClickRandomName = () => 
      (document.getElementById("name-input")! as HTMLInputElement).value = randomName()
    joinGameArea = (
      <>
        <div className="name-input-area">
          <input id="name-input" placeholder="Name"></input>
          <button onClick={onClickJoin}>Join Game</button>
          <button onClick={onClickRandomName}>Random Name</button>
        </div>
      </>
    )
  }

  let playerList
  if (game.players.length == 0) {
    playerList = <h3>None yet</h3>
  } else {
    const playerListItems = game.players.map((player, i) => {
      let thisIsMeButton
      let thisIsYou
      if (name === undefined)
        thisIsMeButton = <button onClick={() => joinGame(player.name)}>This is me</button>
      else if (name === player.name)
        thisIsYou = "(you)"
      return <li key={i}>{player.name} {thisIsMeButton} {thisIsYou}</li>
    })
    playerList = <ul className="setup-view-player-list">{playerListItems}</ul>
  }

  const startGameButtonDisabled = game.players.length < 2

  return (
    <div className="setup-view">
      <h1>Game setup</h1>
      {joinGameArea}
      <br />
      <br />
      <button onClick={startGame} disabled={startGameButtonDisabled}>Start The Game</button>
      <h2>Players:</h2>
      {playerList}
    </div>
  )
}
