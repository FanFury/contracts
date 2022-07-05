import axios  from 'axios';
import {readFileSync, writeFileSync} from "fs";
import path from 'path';

const LOC = '.'

export const promiseRequest = async function (methodName, url) {
  switch (methodName) {
    case 'GET':
      return await axios.get(url);
    default:
      console.log("didn't choose any appropriate case!");
      break;
  }
}

export function writeArtifact(data, name = 'alltx') {
  writeFileSync(path.join(LOC, `${name}.json`), JSON.stringify(data, null, 2))
}

export function readArtifact(name = '/home/ashwkuma/for_juno/contracts/testing/txsFetch/alltx.json') {
  try {
      // const data = readFileSync(path.join(LOC, `${name}.json`), 'utf8')
      const data = readFileSync(name, 'utf8')
      return JSON.parse(data)
  } catch (e) {
      return {}
  }
}

export function IsNotNullAndUndefined (obj) {
  return (typeof obj !== 'undefined' && obj);
}



