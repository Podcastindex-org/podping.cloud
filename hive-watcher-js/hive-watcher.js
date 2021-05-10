const addressList = [
    'https://api.openhive.network',
    'https://api.hive.blog',
    'https://hive.roelandp.nl',
    'https://techcoderx.com',
    'https://api.hivekings.com',
    'https://anyx.io',
    'https://api.deathwing.me',
    'https://hive-api.arcange.eu',
    'https://rpc.ecency.com',
    'https://hived.privex.io',
]
const address = addressList[Math.floor(Math.random() * (addressList.length - 1))]
console.log(`Using endpoint "${address}". If not loading, refresh page to select a different address`)
const client = new dhive.Client(address);

let validAccounts = ['podping']

function handlePost(post) {
    let operationType = post.op[0]
    if (operationType !== 'custom_json')
        return

    let operation = post.op[1]
    if (operation.id !== 'podping')
        return;

    let postJson = JSON.parse(operation.json)

    let postingAuths = new Set(operation.required_posting_auths)

    // check if valid user
    let accounts = new Set(validAccounts)
    let intersect = new Set()
    for (let x of accounts) {
        if (postingAuths.has(x))
            intersect.add(x);
    }
    if (intersect.size === 0)
        return;

    let block = post.block
    let timestamp = post.timestamp
    let trx_id = post.trx_id

    const message = `Feed updated - ${timestamp} - ${block} - ${trx_id} - ${postJson["url"]}`;
    console.log(message)

    const list = document.getElementById("posts");
    const item = document.createElement("li");
    item.appendChild(document.createTextNode(message))
    list.appendChild(item)
}

client.database.call('get_following', [validAccounts[0], null, 'blog', 10])
    .then(function (followers) {
            for (let follower of followers) {
                validAccounts = validAccounts.concat(follower.following)
            }
        }
    )
    .then(function () {
            // can pass the block number to start searching from. By default, uses the current block
            // note: using mode BlockchainMode.Latest does not seem to return data using `getOperationsStream`
            client.blockchain.getOperationsStream({mode: dhive.BlockchainMode.Irreversible})
                .on('data', handlePost);

        }
    );
