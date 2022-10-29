/**
 * Example way to use the dhive library to watch for podpings on the Hive blockchain.
 */

MEDIUM_PODCAST = "podcast"
MEDIUM_AUDIOBOOK = "audiobook"
MEDIUM_BLOG = "blog"
MEDIUM_FILM = "film"
MEDIUM_MUSIC = "music"
MEDIUM_NEWSLETTER = "newsletter"
MEDIUM_VIDEO = "video"

PodpingMedium = [
    MEDIUM_PODCAST,
    MEDIUM_AUDIOBOOK,
    MEDIUM_BLOG,
    MEDIUM_FILM,
    MEDIUM_MUSIC,
    MEDIUM_NEWSLETTER,
    MEDIUM_VIDEO,
]

REASON_LIVE = "live"
REASON_LIVE_END = "liveEnd"
REASON_UPDATE = "update"

PodpingReason = [
    REASON_LIVE,
    REASON_LIVE_END,
    REASON_UPDATE,
]

let lastBlockNumber = undefined

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
    "https://api.openhive.network",
    "https://api.hive.blog",
    "https://anyx.io",
    "https://api.deathwing.me",
]
// shuffle list to try and spread the load across different endpoints since the first item is always used.
shuffleArray(addressList)
console.log(`Using API address "${addressList[0]}"`)
const client = new dhive.Client(
    addressList, {
        // reducing timeout and threshold makes fall over to other API address happen faster
        timeout: 6000, // in ms
        failoverThreshold: 1,
    }
);

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
        console.error("error handling block")
        console.error(error)
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
    lastBlockNumber = blockNumber

    const block_num_span = document.getElementById("block_num");
    block_num_span.innerText = blockNumber.toLocaleString()

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
    if (post.id === "podping" || post.id.startsWith("pp_")) {
        handlePodpingPost(post, timestamp, blockNumber, transactionId)
    }
    // To include test posts
    // else if (post.id === "podpingtest" || post.id.startsWith("pplt_")) {
    //     handlePodpingPost(post, timestamp, blockNumber, transactionId)
    // }
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

    let version = postJson.version || postJson.v
    let updateReason = postJson.reason || postJson.r || postJson.type
    let medium = postJson.medium

    if (version === "1.0") {
        if (!(PodpingReason.includes(updateReason) && PodpingMedium.includes(medium))) {
            return
        }
    } else {
        // fallback to any possible
        // old posts didn't include an update type so still accept them
        if (updateReason !== undefined && updateReason !== "feed_update" && updateReason !== 1)
            return
    }

    let iris = postJson.iris || []
    let urls = postJson.urls || []
    if (urls) {
        iris = iris.concat(urls)
    }
    if (postJson.url) {
        iris = [postJson.url]
    }

    const list = document.getElementById("posts");

    // add ping
    const transactionMessage = `Feed updated(s) - ${timestamp} - ${blockNumber} - ${transactionId} - ${updateReason} - ${medium}`;
    console.log(transactionMessage)

    const item = document.createElement("li");
    item.appendChild(document.createTextNode(transactionMessage))
    list.appendChild(item)

    const subList = document.createElement("ul")
    item.appendChild(subList)
    for (let iri of iris) {
        console.log(`  - ${iri}`)

        const subItem = document.createElement("li");

        const linkItem = document.createElement("a")
        linkItem.href = iri
        linkItem.target = "_blank"
        linkItem.appendChild(document.createTextNode(iri))
        subItem.appendChild(linkItem)

        subList.appendChild(subItem)
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

client.database.call('get_following', [validAccounts[0], null, 'blog', 100])
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
        startStream
    )
;

function startStream(blockNumber = undefined) {
    // can pass the block number to start searching from. By default, uses the current block
    // note: using mode BlockchainMode.Latest does not seem to return data using `getOperationsStream` so
    // `getBlockStream` is used instead and transactions are parsed by block
    client.blockchain.getBlockStream({
        from: blockNumber,
        mode: dhive.BlockchainMode.Latest
    })
        .on('data', handleBlock)
        .on('error',
            function (error) {
                console.error('Error occurred parsing stream')
                console.error(error)
                // Note: when an error occurs, the `end` event is emitted (see below)
            }
        )
        .on('end',
            function () {
                console.log('Reached end of stream')
                // Note: this restart the stream
                startStream(lastBlockNumber);
            }
        );

}
