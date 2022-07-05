import config from './config.js'
import { IsNotNullAndUndefined, promiseRequest, readArtifact, writeArtifact } from './util.js'

const acMap = new Map()
// const spenderList = []
let txList = []
let furyBList = []
let uusdList = []

const UUSD = 'uusd'
const FURY = 'fury'
const AC_WITHDRAW_FURY = "withdraw_fury"
const AC_WITHDRAW_UUSD = "withdraw_uusd"
const AC_DEPOSIT = "deposit"


const userTxMap = {}
const userBalanceList = []

async function checkTxList() {
  for (const tx of config.values()) {
    let url = "https://columbus-fcd.terra.dev/v1/tx/" + tx
    let resp = await promiseRequest("GET", url)
    txList.push(resp.data)
  }
  return txList
}

function createUserTxInfo() {
  txList = readArtifact("alltx.json")
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
      if (IsNotNullAndUndefined(elem.value.execute_msg.withdraw_tokens)) {
        action = AC_WITHDRAW_FURY
      }
      if(IsNotNullAndUndefined(elem.value.execute_msg.withdraw)) {
        action = AC_WITHDRAW_UUSD
      }
    }
  })
  return action
}

function processAction(tx, action) {
  try {
    if (action === AC_WITHDRAW_FURY) {
      processTxWithdrawFury(tx, action)
    } else if (action === AC_WITHDRAW_UUSD) {
      processTxWithdrawUUSD(tx, action)
    } else if (action === AC_DEPOSIT) {
      processTxDepost(tx, action)
    }
  } catch (e) {
    console.error("failed for txhash : " + tx.txhash + " , message : " + tx.raw_log)
    //console.log(e)
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
      updateUserTx(tx, userTx, type, amount, action)
    }
  })
}

function processTxWithdrawUUSD(tx, action) {
  let userTx = userTxDetail()
  let type, amount
  tx.tx.value.msg.forEach(elem => {
    if (elem.type === "wasm/MsgExecuteContract") {
      let type = UUSD
      let amount
      if (IsNotNullAndUndefined(elem.value.execute_msg.withdraw) && IsNotNullAndUndefined(elem.value.execute_msg.withdraw.amount)) {
        amount = elem.value.execute_msg.withdraw.amount
      }  else {
        amount = '-1111111111111111111111111'
      }
      userTx.user = elem.value.sender
      updateUserTx(tx, userTx, type, amount, action)
    }
  })
}

function processTxWithdrawFury(tx, action) {
  let userTx = userTxDetail()
  tx.logs[0].events.forEach(evt => {
    if (evt.type === 'wasm') {
      let amount
      let type = FURY
      evt.attributes.forEach(attribs => {
        if (attribs.key === 'to') {
          userTx.user = attribs.value
        }
        if (attribs.key === 'amount') {
          amount = attribs.value
        }
      })
      updateUserTx(tx, userTx, type, amount, action)
    }
  })
}

function updateUserTx(tx, userTx, type, amount, action) {
  if (IsNotNullAndUndefined(userTxMap[userTx.user])) {
    userTx = userTxMap[userTx.user]
  }
  userTx.txHash.push(tx.txhash)
  populateUserTx(userTx, amount, type, action)
  userTxMap[userTx.user]= userTx
}


function populateUserTx(userTx, amount, type, action) {
  if (action === AC_WITHDRAW_FURY || action === AC_WITHDRAW_UUSD) {
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

function processUserTxs() {
  
  // let uTxMap = readArtifact("/home/ashwkuma/for_juno/contracts/testing/txsFetch/userTx.json")
  let uTxMap = readArtifact("userTx.json")
  Object.values(uTxMap).forEach(elem => {
    let uBal = userBalance()
    uBal.user = elem.user
    
    let amount = getDepositWithdrawal(elem.furyTx)
    uBal.furyBalance = amount.deposited - amount.withdrawal
    amount = getDepositWithdrawal(elem.uusdTx)
    uBal.uusdBalance = amount.deposited - amount.withdrawal
    userBalanceList.push(uBal)
  })
}

function getDepositWithdrawal(txType) {
  let deposited = 0
  let withdrawal = 0
  txType.deposit.forEach(dpAmount => {
    deposited += parseInt(dpAmount)
  })
  txType.withdrawal.forEach(wAmount => {
    withdrawal += parseInt(wAmount)
  })
  return { deposited: deposited, withdrawal: withdrawal }
}

async function processTxs() {
  // For fetching all txs from forge contract. All Tx are in completetx.json
  // await checkTxList()
  // await writeArtifact(txList)
  
  // Process all txs existing in alltx.json & crcreate userTx.json containing details of userTx as per userTxDetail structure
  await createUserTxInfo()
  writeArtifact(userTxMap, "userTx")

  processUserTxs()
  let count = 1
  userBalanceList.forEach(elem => {
    console.log(count++ + "." + elem.user + " : furyBalance : " + elem.furyBalance + " , uusdBalance : " + elem.uusdBalance )
  })
  console.log("completed..")
}


const userBalance = () => {
  return {
    "user": "",
    "furyBalance": "",
    "uusdBalance": ""
  }
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
    },
    "txHash" : []
  }
}


processTxs()