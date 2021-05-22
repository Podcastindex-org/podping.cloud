/**
 * Example way to use the dhive library to watch for podpings on the Hive blockchain.
 */


/**
 * Shuffles items in an array. Shuffles in place
 *
 * from https://stackoverflow.com/a/12646864
 *
 * @param array list of items to shuffle
 */
function shuffleArray(array) {
    for (let i = array.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [array[i], array[j]] = [array[j], array[i]];
    }
}

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
// shuffle list to try and spread the load across different endpoints since the first item is always used.
shuffleArray(addressList)
console.log(`Using API address "${addressList[0]}"`)
const client = new dhive.Client(addressList, {
    // reducing timeout and threshold makes fall over to other API address happen faster
    timeout: 2000, // in ms
    failoverThreshold: 1,
});

const VALID_CUSTOM_JSON_ID = "podping"

let validAccounts = ['podping']

/**
 * Handle new block received
 *
 * @param block received block
 */
function handleBlock(block) {
    try {
        let timestamp = block.timestamp

        for (let transaction of block.transactions) {
            handleTransaction(transaction, timestamp)
        }
    } catch (error) {
        console.log("error handling block")
        console.log(error)
    }
}

/**
 * Handle transaction included in block
 *
 * @param transaction received transaction
 * @param timestamp timestamp for block transaction is in
 */
function handleTransaction(transaction, timestamp) {
    let blockNumber = transaction.block_num
    let transactionId = transaction.transaction_id

    for (let operation of transaction.operations) {
        handleOperation(operation, timestamp, blockNumber, transactionId)
    }
}

/**
 * Handle operation included in transaction
 *
 * @param operation received operation
 * @param timestamp timestamp for block operation is in
 * @param blockNumber block number operation is in
 * @param transactionId transaction operation is in
 */
function handleOperation(operation, timestamp, blockNumber, transactionId) {
    let operationType = operation[0]
    if (operationType === "custom_json") {
        let post = operation[1]
        handleCustomJsonPost(post, timestamp, blockNumber, transactionId)
    }
}

/**
 * Handle custom JSON post and check if desired type
 *
 * @param post received post in operation
 * @param timestamp timestamp for block post is in
 * @param blockNumber block number post is in
 * @param transactionId transaction identifier post is in
 */
function handleCustomJsonPost(post, timestamp, blockNumber, transactionId) {
    if (post.id === VALID_CUSTOM_JSON_ID) {
        handlePodpingPost(post, timestamp, blockNumber, transactionId)
    }
}


/**
 * Handle PodPing post
 *
 * @param post received post in operation
 * @param timestamp timestamp for block post is in
 * @param blockNumber block number post is in
 * @param transactionId transaction identifier post is in
 */
function handlePodpingPost(post, timestamp, blockNumber, transactionId) {
    if (!this.isAccountAllowed(post.required_posting_auths))
        return

    let postJson = JSON.parse(post.json)

    // only accept feed update types
    let updateReason = postJson.reason
    // old posts didn't include an update type so still accept them
    if (updateReason !== undefined && updateReason !== "feed_update")
        return

    let urls = postJson.urls
    if (urls === undefined) {
        urls = [postJson.url]
    }

    const list = document.getElementById("posts");
    for (let url of urls) {
        // add ping
        const message = `Feed updated - ${timestamp} - ${blockNumber} - ${transactionId} - ${url}`;
        console.log(message)

        const item = document.createElement("li");
        item.appendChild(document.createTextNode(message))
        list.appendChild(item)

    }
}

/**
 * Checks if account making PodPing is allowed
 *
 * @param required_posting_auths account used to make PodPing post
 * @returns true if account is valid
 */
function isAccountAllowed(required_posting_auths) {
    // check if valid user
    let postingAuths = new Set(required_posting_auths)
    let accounts = new Set(validAccounts)
    let intersect = new Set()
    for (let x of accounts) {
        if (postingAuths.has(x))
            intersect.add(x)
    }
    // if accounts don't overlap, skip post
    return intersect.size !== 0
}

client.database.call('get_following', [validAccounts[0], null, 'blog', 10])
    .then(
        /**
         * Get all accounts that are accepted as a valid PodPing poster
         *
         * @param followers list of follower objects
         */
        function (followers) {
            for (let follower of followers) {
                validAccounts = validAccounts.concat(follower.following)
            }
        }
    )
    .then(
        function () {
            // can pass the block number to start searching from. By default, uses the current block
            // note: using mode BlockchainMode.Latest does not seem to return data using `getOperationsStream` so
            // `getBlockStream` is used instead and transactions are parsed by block
            client.blockchain.getBlockStream({mode: dhive.BlockchainMode.Latest})
                .on('data', handleBlock)
                .on('error',
                    function (error) {
                        console.log('Error occurred parsing stream')
                        console.log(error)
                        // Note: when an error occurs, the `end` event is emitted (see below)
                    }
                )
                .on('end',
                    function () {
                        console.log('Reached end of stream')
                        // Note: could add a stream restart here
                    }
                );
        }
    );
