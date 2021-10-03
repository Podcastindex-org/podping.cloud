import asyncio
import json
import logging

from datetime import datetime, timedelta
from ssl import OP_SINGLE_DH_USE
from timeit import default_timer as timer
from typing import Any, List, Optional

from privex.steem import SteemAsync
from privex.steem.objects import Block, Operation
from pydantic import BaseModel
from pydantic.fields import Field


# Experimental Async version of the podping-watcher
# Brian of London


log = logging.getLogger('privex.steem')
log.level = logging.ERROR
log.setLevel(logging.ERROR)


class PodpingJson(BaseModel):
    version: str
    num_urls: int
    reason: str
    urls: List[str]


class PodpingData(BaseModel):
    required_auths: List[str]
    required_posting_auths: List[str]
    id: str
    payload: PodpingJson = Field(alias="json")


class PodpingOp(BaseModel):
    op_txid: str
    op_type: str
    op_block_num: int
    timestamp: Optional[datetime]
    age: Optional[timedelta]
    data: PodpingData

    def __init__(__pydantic_self__, **data: Any) -> None:
        if type(data["data"]["json"]) == str:
            data["data"]["json"] = PodpingJson.parse_raw(
                data["data"]["json"]
            )
        # if type(data["timestamp"]) == str:
        #     data["timestamp"] = datetime.strptime(data["timestamp"], "%Y-%m-%dT%H:%M:%S")
        super().__init__(**data)


class OpFiltered(BaseModel, Operation):
    pass


rpc_nodes = [
    "https://api.deathwing.me",
    "https://hive-api.3speak.tv",
    "https://hived.emre.sh",
    "https://rpc.ausbit.dev",
    "https://hive-api.arcange.eu",
    "https://api.hive.blog",
    "https://api.openhive.network",
    "https://anyx.io",
    "https://hive-api.3speak.tv",
]


async def main():
    tasks = []
    try:
        s = SteemAsync(rpc_nodes=rpc_nodes)
        stream = s.stream_blocks(before=100, end_after=0, wait_block=2)
        while True:
            try:
                async for b in stream:
                    asyncio.create_task(find_all_podpings(b))

            except KeyboardInterrupt:
                raise KeyboardInterrupt
            except asyncio.CancelledError:
                raise asyncio.CancelledError

    except KeyboardInterrupt as ex:
        s.stop_streaming()
        return

    except asyncio.CancelledError as ex:
        s.stop_streaming()
        return


async def find_all_podpings(b: Block):
    for post in b.transactions:
        for op in post.operations:
            my_op = OpFiltered.parse_obj(op)
            if op.op_type == "custom_json":
                id = op.data.get("id")
                # print(id)

                if id.startswith("podping"):
                    pp = PodpingOp.parse_obj(op)
                    pp.timestamp = datetime.strptime(b.timestamp, "%Y-%m-%dT%H:%M:%S")
                    pp.age = datetime.utcnow() - pp.timestamp
                    print(f"{pp.age} | {pp.op_txid}")
                    for url in pp.data.payload.urls:
                        print(f"--> {url}")




asyncio.run(main())
