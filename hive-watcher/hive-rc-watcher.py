# Code to watch an account's RC level

from beem import Hive
from beem.account import Account


hive = Hive()

account = Account('hivehydra',blockchain_instance=hive,full=True)

print(account)