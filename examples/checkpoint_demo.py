# Continuum explicit safepoint demo (Python).
# Capture UES at checkpoint, resume post-checkpoint region only.
#
#   plx continuum examples/checkpoint_demo.py --resume -o /tmp/demo.ues.json --json

x = 1
y = ["a", "b"]
parallax.checkpoint("after_init")
x = x + 41
y = y + ["c"]
print(f"resumed x={x} y={y}")
