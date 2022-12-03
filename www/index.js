import { generate_universe, Universe } from "rust-wasm-tutorial-conways-game-of-life";

const pre = document.getElementById("game-of-life-canvas");
const universe = generate_universe(48, 48);

const renderLoop = () => {
	pre.textContent = universe.render();
	universe.tick();

	requestAnimationFrame(renderLoop);
};

requestAnimationFrame(renderLoop);
