// import data from "./filtered-before-generate.json" assert { type: 'json' };

import Jimp from "jimp";
import { readFileSync } from "fs";


interface DataFile {
    resource: LuaResource[],
    tile: LuaTile[],
}

interface LuaResource {
    type: string,
    name: string,
    position: Position,
}

interface LuaTile {
    name: string,
    position: Position,
}

interface Position {
    x: number,
    y: number
}

const data: DataFile = JSON.parse(readFileSync("./filtered-before-generate.json", "utf-8"));

let max_x = 0;
let max_y = 0;
for (const tile of data.resource) {
    max_x = Math.max(max_x, tile.position.x);
    max_y = Math.max(max_y, tile.position.y);
}
console.log(`Found max ${max_x}x${max_y}`)
max_x = Math.round(max_x)
max_y = Math.round(max_y)
console.log(`Rounded ${max_x}x${max_y}`)

new Jimp(max_x, max_y, (error, img) => {
    if (error) {
        console.log("jimp crash", error)
        return;
    }
})
