import init, { ThrongletsWeb } from "./pkg/thronglets.js";

const canvas = document.getElementById("world");
const ctx = canvas.getContext("2d");
const statusEl = document.getElementById("status");
const popEl = document.getElementById("pop");
const eggsEl = document.getElementById("eggs");
const foodUnitsEl = document.getElementById("food-units");
const scarcityEl = document.getElementById("scarcity");
const fadedEl = document.getElementById("faded");
const ideasEl = document.getElementById("ideas");
const dayEl = document.getElementById("day");
const badgePopEl = document.getElementById("badge-pop");
const speedEl = document.getElementById("speed");

let game;
let paused = false;
let speed = 1;
let lastStepAt = 0;

const STEP_MS = 66;

function draw(now = performance.now()) {
  if (!game) return;
  if (!lastStepAt) lastStepAt = now;
  if (!paused) {
    const dueSteps = Math.min(Math.floor((now - lastStepAt) / STEP_MS), 4);
    if (dueSteps > 0) {
      for (let i = 0; i < dueSteps; i += 1) game.step(speed);
      lastStepAt += dueSteps * STEP_MS;
    }
  } else {
    lastStepAt = now;
  }

  const rgba = game.render_rgba(canvas.width, canvas.height);
  const image = new ImageData(new Uint8ClampedArray(rgba), canvas.width, canvas.height);
  ctx.putImageData(image, 0, 0);

  popEl.textContent = game.population();
  badgePopEl.textContent = game.population();
  eggsEl.textContent = game.eggs();
  foodUnitsEl.textContent = game.food_units();
  scarcityEl.textContent = `${Math.round(game.scarcity() * 100)}%`;
  fadedEl.textContent = game.faded();
  ideasEl.textContent = game.ideas();
  dayEl.textContent = Math.floor(Number(game.tick()) / 2400);
  speedEl.textContent = speed;
  statusEl.textContent = `${paused ? "paused" : "running"} | ${game.theme_name()}`;

  requestAnimationFrame(draw);
}

function bindControls() {
  document.getElementById("food").addEventListener("click", () => game.drop_food());
  document.getElementById("egg").addEventListener("click", () => game.place_egg());
  document.getElementById("idea").addEventListener("click", () => game.seed_idea());
  document.getElementById("theme").addEventListener("click", () => game.next_theme());

  canvas.addEventListener("pointerdown", (event) => {
    const rect = canvas.getBoundingClientRect();
    const x = game.camera_x() + Math.floor(((event.clientX - rect.left) / rect.width) * game.view_width());
    const y = game.camera_y() + Math.floor(((event.clientY - rect.top) / rect.height) * game.view_height());
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
    statusEl.textContent = "missing wasm bundle";
  }
}

main();
