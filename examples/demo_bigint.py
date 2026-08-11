# Demonstrates integer precision policy: value outside JS Number safe range.
state = {
    "username": "Ada",
    "score": 9007199254740993,  # Number.MAX_SAFE_INTEGER + 2
}
