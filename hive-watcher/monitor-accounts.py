# Simple code to loop through accounts of interest and report their
# Resources


from beem import Hive
from beem.account import Account
import time

accs = [
    "podping",
    "podping.aaa",
    "podping.bbb",
    "podping.ccc",
    "podping.spk",
    "podping.test",
    "podping.gittest",
    "podping.bol",
    "brianoflondon",
    "v4vapp",
]


def main():

    last_result = []
    new_result = []
    for acc in accs:
        mana_bar = Account(acc).get_rc_manabar()
        print(f"{acc:<16} {mana_bar['current_pct']:>5.1f}")
    time.sleep(5*60)


if __name__ == "__main__":
    main()
