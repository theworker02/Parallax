// Continuum explicit safepoint demo (JavaScript).
// Prefer `var` (or pass locals to checkpoint) so bindings attach to the sandbox.
//
//   plx continuum examples/checkpoint_demo.js --resume -o /tmp/demo.ues.json --json

var x = 1;
var y = ["a", "b"];
parallax.checkpoint("after_init", { x: x, y: y });
x = x + 41;
y = y.concat(["c"]);
console.log("resumed x=" + x + " y=" + JSON.stringify(y));
