import { useState } from "react"
import { TileBar } from "./gameview/TileBar"
import { TileT } from "./game-types"

export const DebugInfo = ({ data }: { data: Record<string, unknown> }) => {
  const [show, setShow] = useState(false)
  const white = { color: "white" }
  const blacklist = ["board", "tile_bag", "tiles"]

  let debugPane
  if (show) {
    const stringify = (x: unknown) => {
      return JSON.stringify(x, (k, v) => blacklist.includes(k) ? "(redacted)" : v, 1)
    }

    const listItems = Object.entries(data).map(([label, val], i) =>
      <li key={i} style={white}>
        <pre style={white}>
          {label}: {stringify(val)}
        </pre>
      </li>)
    const list = <ul style={{ border: "1px solid white", borderRadius: "3px" }}>{listItems}</ul>

    debugPane = (
      <>
        {list}
        <TileBar tiles={["A", "B", "C", "D", "E", "F", "G"]} onClickTile={() => {}} selectedTile={undefined} />
        <TileBar tiles={["H", "I", "J", "K", "L", "M", "N"]} onClickTile={() => {}} selectedTile={undefined} />
        <TileBar tiles={["O", "P", "Q", "R", "S", "T", "U"]} onClickTile={() => {}} selectedTile={undefined} />
        <TileBar tiles={["V", "W", "X", "Y", "Z", "Blank", { "Blank": "X" } as unknown as TileT]} onClickTile={() => {}} selectedTile={undefined} />
      </>
    )
  }

  return (
    <>
      <button onClick={() => setShow(!show)}>debug info</button>
      {debugPane}
    </>
  )
}
