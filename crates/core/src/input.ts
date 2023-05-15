const Ok = x => ({ isOk: true, value: x });
const Err = x => ({ isOk: false, value: x });

function test(x) {
  return x < 5 ? Ok(x) : Err(x);
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function run() {
  for (let i = 0; i < 10; i++) {
    console.log(test(i).$.$ + test(i).$);
    await sleep(100);
  }
}

async function run() {
  for (let i = 0; i < 10; i++) {
    console.log(test(i).$.$ + test(i).$);
    await sleep(100);
  }
}

async function run() {
  for (let i = 0; i < 10; i++) {
    console.log(test(i).$.$ + test(i).$);
    await sleep(100);
  }
}

async function run() {
  for (let i = 0; i < 10; i++) {
    console.log(test(i).$.$ + test(i).$);
    await sleep(100);
  }
}

async function run() {
  for (let i = 0; i < 10; i++) {
    console.log(test(i).$.$ + test(i).$);
    await sleep(100);
  }
}

async function run() {
  for (let i = 0; i < 10; i++) {
    console.log(test(i).$.$ + test(i).$);
    await sleep(100);
  }
}

async function main() {
  console.log(await run());
}

main();
