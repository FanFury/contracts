
import {
    storeCode,
    migrateContract
} from "./utils.js";

let current_address = "terra1rws5tqe6fxl3hgmvywq76c6200rpqsy5tqvyuy"

export function sleep(time) {
    return new Promise((resolve) => setTimeout(resolve, time));
}

