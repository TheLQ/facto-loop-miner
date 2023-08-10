import {readFileSync} from "fs";

export interface DataFile {
    readonly resource: LuaResource[],
    readonly tile: LuaTile[],
    readonly resource_box: EasyBox,
    readonly tile_box: EasyBox,
}

export interface LuaResource {
    readonly type: string,
    readonly name: string,
    readonly position: Position,
}

export interface LuaTile {
    readonly name: string,
    readonly position: Position,
}

export interface Position {
    readonly x: number,
    readonly y: number
}

export interface EasyBox {
    readonly max_x: number
    readonly max_y: number
    readonly min_x: number
    readonly min_y: number
}

type Writeable<T> = { -readonly [P in keyof T]: T[P] };

export function openData(path: string): DataFile {
    const data: Writeable<DataFile> = JSON.parse(readFileSync(path, "utf-8"));
    data.resource_box = build_box(data.resource)
    data.tile_box = build_box(data.tile)

    return data;
}

function build_box(items: LuaTile[] | LuaResource[]): EasyBox {
    const box: Writeable<EasyBox> = {
        max_x: 0,
        max_y: 0,
        min_x: 0,
        min_y: 0,
    }
    for (const item of items) {
        box.max_x = Math.max(box.max_x, item.position.x)
        box.max_y = Math.max(box.max_y, item.position.y)
        box.min_x = Math.min(box.min_x, item.position.x)
        box.min_y = Math.min(box.min_y, item.position.y)
    }
    return box;
}
