import { WasmChip } from "yace";
import { memory } from "yace/yace_bg.wasm";
import "./style.css";

const WIDTH = 64;
const HEIGHT = 32;
const TIMERS_REFRESH_RATE = 60;
const CLOCK_SPEED = 500;
const CELL_SIZE = 6;

const ROMS = ["chip8", "star", "pong", "space-invaders"];

interface Keymap {
  [key: string]: number | undefined;
}

const KEYMAP: Keymap = {
  Digit1: 0x1,
  Digit2: 0x2,
  Digit3: 0x3,
  Digit4: 0xc,
  KeyQ: 0x4,
  KeyW: 0x5,
  KeyE: 0x6,
  KeyR: 0xd,
  KeyA: 0x7,
  KeyS: 0x8,
  KeyD: 0x9,
  KeyF: 0xe,
  KeyZ: 0xa,
  KeyX: 0x0,
  KeyC: 0xb,
  KeyV: 0xf,
};

const chip = new WasmChip();

const displayBuffer = new Uint8Array(
  memory.buffer,
  chip.ptr_display_buffer(),
  WIDTH * HEIGHT
);

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d") as CanvasRenderingContext2D;

let clockInterval: number;
let timersInterval: number;

const init = () => {
  const button = document.getElementById("btn-start");
  const select = document.getElementById("select-rom") as HTMLSelectElement;

  for (const rom of ROMS) {
    const option = document.createElement("option");

    option.value = rom;
    option.innerText = rom;
    select.appendChild(option);
  }

  button?.addEventListener("click", async () => {
    const rom = await getRom(select.value);

    clearInterval(clockInterval);
    clearInterval(timersInterval);

    chip.reset();
    chip.load(rom);

    run();
  });

  canvas.width = WIDTH * CELL_SIZE;
  canvas.height = HEIGHT * CELL_SIZE;

  document.addEventListener("keydown", (event) => {
    const key = KEYMAP[event.code];
    if (key) {
      chip.set_key(key);
    }
  });

  document.addEventListener("keyup", (event) => {
    const key = KEYMAP[event.code];
    if (key === chip.get_key()) {
      chip.unset_key();
    }
  });
};

const getRom = async (name: string) => {
  const rom = await fetch(`/roms/${name}`);
  const buf = await rom.arrayBuffer();
  const bytes = new Uint8Array(buf);

  return bytes;
};

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

const run = async () => {
  clockInterval = setInterval(() => {
    chip.tick();

    if (chip.has_display_changes()) {
      render();
    }
  }, 1000 / CLOCK_SPEED);

  timersInterval = setInterval(
    () => chip.update_timers(),
    1000 / TIMERS_REFRESH_RATE
  );
};

init();
