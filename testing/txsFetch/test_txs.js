import config from './config.js'
import { IsNotNullAndUndefined, promiseRequest, readArtifact, writeArtifact } from './util.js'

const acMap = new Map()
// const spenderList = []
let txList = []
let furyBList = []
let uusdList = []

const UUSD = 'uusd'
const FURY = 'fury'
const AC_WITHDRAW = "withdraw"
const AC_DEPOSIT = "deposit"


const userTxMap = {}

async function checkTxList() {
  for (const tx of config.values()) {
    let url = "https://columbus-fcd.terra.dev/v1/tx/" + tx
    let resp = await promiseRequest("GET", url)
    txList.push(resp.data)
  }
  return txList
}

function createUserTxInfo() {
  txList = readArtifact()
  txList.forEach(tx => {
    let action = checkActionType(tx.tx.value.msg)
    processAction(tx, action)
  });
}


function checkActionType(msgs) {
  let action
  msgs.forEach(elem => {
    if (elem.type === "wasm/MsgExecuteContract") {
      if (IsNotNullAndUndefined(elem.value.execute_msg.deposit)) {
        action = AC_DEPOSIT
      }
      if (IsNotNullAndUndefined(elem.value.execute_msg.withdraw_tokens || elem.value.execute_msg.withdraw)) {
        action = AC_WITHDRAW
      }
    }
  })
  return action
}

function processAction(tx, action) {
  if (action === AC_WITHDRAW) {
    processTxWithdraw(tx.logs[0], action)
  } else if (action === AC_DEPOSIT) {
    processTxDepost(tx, AC_DEPOSIT)
  }

}

function processTxDepost(tx, action) {
  let userTx = userTxDetail()
  let type, amount
  tx.tx.value.msg.forEach(elem => {
    if (elem.type === "wasm/MsgExecuteContract") {
      if (IsNotNullAndUndefined(elem.value.coins)) {
        elem.value.coins.forEach(coin => {
          if (coin.denom === UUSD) {
            type = UUSD
            amount = coin.amount
          } else if (coin.denom === FURY) {
            type = FURY
            amount = coin.amount
          }
        })
      }
      userTx.user = elem.value.sender
      if (IsNotNullAndUndefined(userTxMap[userTx.user])) {
        userTx = userTxMap[userTx.user]
      }
      populateUserTx(userTx, amount, type, action)
      userTxMap[userTx.user]= userTx
    }
  })
}

function processTxWithdraw(log, action) {
  let userTx = userTxDetail()
  log.events.forEach(evt => {
    if (evt.type === 'wasm') {
      let amount
      let type = UUSD;
      evt.attributes.forEach(attribs => {
        if (attribs.key === 'to') {
          userTx.user = attribs.value
        }
        if (attribs.key === 'action' && attribs.value === 'withdraw_tokens') {
          type = FURY
        }
        if (attribs.key === 'action' && attribs.value === 'withdraw') {
          type = UUSD
        }
        if (attribs.key === 'amount') {
          amount = attribs.value
        }
      })
      if (IsNotNullAndUndefined(userTxMap[userTx.user])) {
        userTx = userTxMap[userTx.user]
      }
      populateUserTx(userTx, amount, type, action)
      userTxMap[userTx.user]= userTx
    }
  })
}

function processForWithdrawal() {

}

function populateUserTx(userTx, amount, type, action) {
  if (action === AC_WITHDRAW) {
    if (type === FURY) {
      userTx.furyTx.withdrawal.push(amount)
    } else {
      userTx.uusdTx.withdrawal.push(amount)
    }
  }
  if (action === AC_DEPOSIT) {
    if (type === FURY) {
      userTx.furyTx.deposit.push(amount)
    } else {
      userTx.uusdTx.deposit.push(amount)
    }
  }
}

async function processTxs() {
  // await checkTxList()
  //  Comment it when 
  // await writeArtifact(txList)
  await createUserTxInfo()
  writeArtifact(userTxMap, "userTx")

  console.log("completed")
}




const userTxDetail = () => {
  return {
    "user": "",
    "furyTx": {
      "withdrawal": [],
      "deposit": []
    },
    "uusdTx": {
      "withdrawal": [],
      "deposit": []
    }
  }
}

processTxs()