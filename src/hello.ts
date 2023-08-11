// import data from "./filtered-before-generate.json" assert { type: 'json' };

import Jimp from "jimp";
import {DataFile, LuaResource, openData} from "./data";
import {writeFile} from "fs/promises";
import * as stream from "stream";


async function main() {
    let data = await openData("./filtered-resources2.json")
    const color = Jimp.rgbaToInt(0, 0, 250, 0)
    let img = await createImage(data);
    for (let x = 0; x < 50; x++) {
        for (let y = 0; y < 50; y++) {
            img.setPixelColor(color, x, y)
        }
    }
    fillImage(data, img);
    await writeImg(img, "out.png");
}
process.nextTick(main)

async function createImage(data: DataFile) {
    let length = data.resource_box.max_x - data.resource_box.min_x;
    let height = data.resource_box.max_y - data.resource_box.min_y;
    console.log(`creatimg image ${length}x${height} from`, data.resource_box)
    return await Jimp.create(length, height, "#FFFFFF");
}

function fillImage(data: DataFile, img: Jimp) {
    let pixel_count = 0;
    for (const resource of data.resource) {
        const color = colorForResource(resource);
        if (color == null) {
            continue
        }

        let relative_x = resource.position.x - data.resource_box.min_x;
        let relative_y = resource.position.y - data.resource_box.min_y;
        // console.log(`flip itron ${relative_x}x${relative_y} to ${color}`)
        pixel_count++;
        img.setPixelColor(color, relative_x, relative_y)
    }
    console.log(`set ${pixel_count} pixels fffasdfa`)
}

const loggedUnsupportedTypes: string[] = []
function colorForResource(resource: LuaResource) {
    if (resource.name == "iron-ore") {
        return colorFromHex("#688290")
    } else if (resource.name == "copper-ore") {
        return colorFromHex("#c86230")
    }else if (resource.name == "stone") {
        return colorFromHex("#b09868")
    } else if (resource.name == "coal") {
        return colorFromHex("#000000")
    } else if (resource.name == "uranium-ore") {
        return colorFromHex("#00b200")
    } else {
        if (!loggedUnsupportedTypes.includes(resource.name)) {
            console.log("unsupported " + resource.name)
            loggedUnsupportedTypes.push(resource.name)
        }
        return null
    }
}

function fillImage2(data: DataFile, img: Jimp) {
    img.scan(0, 0, img.bitmap.width, img.bitmap.height, function (x, y, idx) {
        // x, y is the position of this pixel on the image
        // idx is the position start position of this rgba tuple in the bitmap Buffer
        // this is the image

        // var red = this.bitmap.data[idx + 0];
        // var green = this.bitmap.data[idx + 1];
        // var blue = this.bitmap.data[idx + 2];
        // var alpha = this.bitmap.data[idx + 3];
        this.bitmap.data[idx + 1] = 200

        // rgba values run from 0 - 255
        // e.g. this.bitmap.data[idx] = 0; // removes red from this pixel
    });
}

async function writeImg(img: Jimp, path: string) {
    console.log("mime " + img.getMIME())
    img.rgba(false)
    const buf = await img.getBufferAsync(Jimp.MIME_BMP)
    await writeFile(path, buf);
}

function colorFromHex(hexStr: string) {
    return parseInt(hexStr.substring(1), 16)
}
