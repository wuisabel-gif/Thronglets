import init, { ThrongletsWeb } from "./pkg/thronglets.js";

const canvas = document.getElementById("world");
const ctx = canvas.getContext("2d");
const statusEl = document.getElementById("status");
const popEl = document.getElementById("pop");
const eggsEl = document.getElementById("eggs");
const fadedEl = document.getElementById("faded");
const ideasEl = document.getElementById("ideas");

let game;
let paused = false;
let speed = 1;

function draw() {
  if (!game) return;
  if (!paused) game.step(speed);

  const rgba = game.render_rgba(canvas.width, canvas.height);
  const image = new ImageData(new Uint8ClampedArray(rgba), canvas.width, canvas.height);
  ctx.putImageData(image, 0, 0);

  popEl.textContent = game.population();
  eggsEl.textContent = game.eggs();
  fadedEl.textContent = game.faded();
  ideasEl.textContent = game.ideas();
  statusEl.textContent = `${paused ? "Paused" : "Running"} · ${game.theme_name()} · x${speed}`;

  requestAnimationFrame(draw);
}

function bindControls() {
  document.getElementById("food").addEventListener("click", () => game.drop_food());
  document.getElementById("egg").addEventListener("click", () => game.place_egg());
  document.getElementById("idea").addEventListener("click", () => game.seed_idea());
  document.getElementById("theme").addEventListener("click", () => game.next_theme());

  canvas.addEventListener("pointerdown", (event) => {
    const rect = canvas.getBoundingClientRect();
    const x = Math.floor(((event.clientX - rect.left) / rect.width) * game.world_width());
    const y = Math.floor(((event.clientY - rect.top) / rect.height) * game.world_height());
    game.set_cursor(x, y);
  });

  window.addEventListener("keydown", (event) => {
    if (!game) return;
    if (event.key === "ArrowLeft") game.move_cursor(-2, 0);
    else if (event.key === "ArrowRight") game.move_cursor(2, 0);
    else if (event.key === "ArrowUp") game.move_cursor(0, -2);
    else if (event.key === "ArrowDown") game.move_cursor(0, 2);
    else if (event.key === "f" || event.key === "F") game.drop_food();
    else if (event.key === "e" || event.key === "E") game.place_egg();
    else if (event.key === "T") game.seed_idea();
    else if (event.key === "t") game.next_theme();
    else if (event.key === " ") paused = !paused;
    else if (event.key === "+" || event.key === "=") speed = Math.min(speed * 2, 16);
    else if (event.key === "-") speed = Math.max(Math.floor(speed / 2), 1);
    else return;
    event.preventDefault();
  });
}

async function main() {
  try {
    await init();
    game = new ThrongletsWeb(1997, 8);
    bindControls();
    draw();
  } catch (error) {
    console.error(error);
    statusEl.textContent = "Build the WASM bundle first: scripts/build-wasm.sh";
  }
}

main();
