import { seedRandomRange } from "./seeded-random"

const paletteDark = ["#F49098", "#CFAD4C", "#59C99E", "#61BBF7", "#CE99E6", "#F59282", "#BDB551", "#3ECAB4", "#7BB4FD", "#DD94D6", "#F1976E", "#A7BC5F", "#2BC9C9", "#93ADFF", "#E891C3", "#EA9E5C", "#8FC272", "#30C6DC", "#A9A6FB", "#F08FAE", "#DEA550", "#75C787", "#46C1EB", "#BD9FF3"]
const paletteLight = ["#A45C63", "#8A722B", "#348667", "#3A7BA6", "#89639A", "#A45E53", "#7D772F", "#1D8676", "#4D76AB", "#93608F", "#A26244", "#6D7C39", "#098585", "#5F71AC", "#9B5D81", "#9D6637", "#5C8147", "#0F8393", "#6F6CA9", "#A15C72", "#956C2E", "#498456", "#25809E", "#7D67A3"]

export function getUserColor(username: string, palette: "dark" | "light") {
  if (palette === "dark") {
    return paletteDark[seedRandomRange(0, paletteDark.length - 1, username)]
  }

  return paletteLight[seedRandomRange(0, paletteDark.length - 1, username)]
}
