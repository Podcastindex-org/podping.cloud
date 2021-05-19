#!/usr/bin/env python3
import os

#creates data.csv file with header row
filepath="data.csv"
if not os.path.isfile(filepath):
    open(filepath, 'w').close()
    content = 'timestamp_seen,timestamp_post,id,trx_id,trx_num,type,block_num,required_auths,required_posting_auths,"json"'
    # Open for appending and write to log
    f = open(filepath, "a")
    f.write(content)
    f.close()
