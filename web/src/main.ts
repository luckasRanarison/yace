import { WasmChip } from "yace";
import { memory } from "yace/yace_bg.wasm";
import "./style.css";

const WIDTH = 64;
const HEIGHT = 32;
const CELL_SIZE = 5;
const STEPS_PER_CYCLE = 10;

const ROMS = [
  "chip8",
  "pong",
  "tank",
  "tetris",
  "tic-tac-toe",
  "space-invaders",
];

const keymap: Map<string, number> = new Map([
  ["Digit1", 0x1],
  ["Digit2", 0x2],
  ["Digit3", 0x3],
  ["Digit4", 0xc],
  ["KeyQ", 0x4],
  ["KeyW", 0x5],
  ["KeyE", 0x6],
  ["KeyR", 0xd],
  ["KeyA", 0x7],
  ["KeyS", 0x8],
  ["KeyD", 0x9],
  ["KeyF", 0xe],
  ["KeyZ", 0xa],
  ["KeyX", 0x0],
  ["KeyC", 0xb],
  ["KeyV", 0xf],
]);

const chip = new WasmChip();

const displayBuffer = new Uint8Array(
  memory.buffer,
  chip.ptr_display_buffer(),
  WIDTH * HEIGHT
);

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d") as CanvasRenderingContext2D;

let currentFrame: number;

const render = () => {
  ctx.clearRect(0, 0, WIDTH * CELL_SIZE, HEIGHT * CELL_SIZE);

  for (let i = 0; i < displayBuffer.length; i++) {
    const row = Math.floor(i / WIDTH);
    const col = i % WIDTH;
    const color = displayBuffer[i] == 1 ? "#FFF" : "#000";

    ctx.fillStyle = color;
    ctx.fillRect(col * CELL_SIZE, row * CELL_SIZE, CELL_SIZE, CELL_SIZE);
  }
};

const loop = () => {
  const executeCycle = () => {
    for (let i = 0; i < STEPS_PER_CYCLE; i++) {
      chip.tick();
    }

    chip.update_timers();
    render();

    currentFrame = requestAnimationFrame(executeCycle);
  };

  requestAnimationFrame(executeCycle);
};

const getRom = async (name: string) => {
  const rom = await fetch(`/yace/roms/${name}`);
  const buf = await rom.arrayBuffer();
  const bytes = new Uint8Array(buf);

  return bytes;
};

const onKeyDown = (key: number | undefined) => {
  if (key != undefined) {
    const keyElem = document.querySelector(`#table-key #key-${key}`);
    keyElem?.classList.add("key-pressed");
    chip.set_key(key);
  }
};

const onKeyUp = (key: number | undefined) => {
  if (key != undefined) {
    const keyElem = document.querySelector(`#table-key #key-${key}`);
    keyElem?.classList.remove("key-pressed");
    chip.unset_key(key);
  }
};

const onTouchDown = (event: Event) => {
  const target = event.target as HTMLElement;
  const key = target.dataset.key;
  if (key) onKeyDown(parseInt(key));
};

const onTouchUp = (event: Event) => {
  const target = event.target as HTMLElement;
  const key = target.dataset.key;
  if (key) onKeyUp(parseInt(key));
};

const init = () => {
  const button = document.getElementById("btn-start");
  const select = document.getElementById("select-rom") as HTMLSelectElement;
  const keys = document.querySelectorAll("#table-key td");

  for (const rom of ROMS) {
    const option = document.createElement("option");

    option.value = rom;
    option.innerText = rom;
    select.appendChild(option);
  }

  select.selectedIndex = 0;

  button?.addEventListener("click", async () => {
    if (select.value) {
      const rom = await getRom(select.value);

      if (currentFrame) {
        cancelAnimationFrame(currentFrame);
      }

      chip.reset();
      chip.load(rom);

      loop();
    }
  });

  for (const keyElem of keys) {
    keyElem.addEventListener("touchstart", onTouchDown);
    keyElem.addEventListener("mousedown", onTouchDown);
    keyElem.addEventListener("touchend", onTouchUp);
    keyElem.addEventListener("mouseup", onTouchUp);
  }

  canvas.width = WIDTH * CELL_SIZE;
  canvas.height = HEIGHT * CELL_SIZE;

  document.addEventListener("keydown", (e) => onKeyDown(keymap.get(e.code)));
  document.addEventListener("keyup", (e) => onKeyUp(keymap.get(e.code)));
};

const fetchRepoData = async () => {
  const res = await fetch("https://api.github.com/repos/luckasRanarison/yace");
  const parsed = await res.json();
  const starElem = document.getElementById("status-star") as HTMLElement;
  const watcherElem = document.getElementById("status-watcher") as HTMLElement;
  const forkElem = document.getElementById("status-fork") as HTMLElement;

  starElem.innerText = parsed.stargazers_count;
  watcherElem.innerText = parsed.forks_count;
  forkElem.innerText = parsed.watchers_count;
};

init();
fetchRepoData();
