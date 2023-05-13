import random
import time
import json


KEY_CHOICE = ["host", "method", "process", "cpu", "time", "action"]
VAL_CHOICE = ["some-host", "some-method", "test", "some-test", "event", "toto", "string", "application", "word", "titi", "tata", "sample", "metric"]


def gen_val():
    r = random.random()
    if r < 0.3:
        return random.randint(0, 10) + random.random()
    if r < 0.6:
        return random.randint(0, 10)
    if r < 0.9:
        return random.choice(VAL_CHOICE)
    if r < 0.95:
        return [gen_val() for _ in range(random.randint(3, 5))]
    d = {}
    gen_rec(d)
    return d


def gen_rec(d):
    for i in range(random.randint(3, 5)):
        k = random.choice(KEY_CHOICE)
        d[k] = gen_val()
    


c = 0
t = time.time()
while True:
    c += 1
    r = random.random()
    if r < 0.9:
        level = "INFO"
    elif r < 0.97:
        level = "WARNING"
    else:
        level = "ERROR"
    d = {"level": level}
    gen_rec(d)
    if c % 3000 == 0:
        t2 = time.time()
        if t2 - t < 0.1:
            time.sleep(0.1 - t2 + t)
        t = t2
    print(json.dumps(d))
