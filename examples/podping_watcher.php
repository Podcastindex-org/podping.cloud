<?php

// Monitor hive-watcher.py output continuously for notifications
// It's expected you would run this script like so:
//    python3 -u ./hive-watcher.py --json | php ./podping_watcher.php
//
//


//Monitor hive-watcher.py output continuously
while (1) {

    sleep(1);

    //Vars
    $timestamp = date(DATE_RFC2822);
    $reason = "update";
    $medium = "podcast";
    $version = "";
    $urls = [];

    //Get the incoming podping json payload from STDIN and parse it
    $json = trim(readline());
    $podping = json_decode($json, TRUE);

    echo "--- $timestamp ---\n";

    //Bail on unknown payload schema
    if (!isset($podping['version']) || empty($podping['version'])) {
        continue;
    }

    //Reason code
    //_https://github.com/Podcastindex-org/podping-hivewriter#podping-reasons
    if (isset($podping['reason']) && !empty($podping['reason'])) {
        $reason = $podping['reason'];
    }

    //Medium code
    //_https://github.com/Podcastindex-org/podping-hivewriter#mediums
    if (isset($podping['medium']) && !empty($podping['medium'])) {
        $medium = $podping['medium'];
    }

    //Get the url list from the payload
    //_https://github.com/Podcastindex-org/podping-hivewriter/issues/26
    switch ($podping['version']) {
        case "0.3":
            $version = "0.3";
            $iris = $podping['urls'];
            break;
        case "1.0":
            $version = "1.0";
            $iris = $podping['iris'];
            break;
        default:
            continue 2;
    }

    //Logging - incoming podping banner
    echo "PODPING(v$version) - $medium - $reason:\n";

    //Handle each iri
    foreach ($iris as $iri) {
        //Make sure it's a valid iri scheme that we are prepared to handle
        if (stripos($iri, 'http://') !== 0
            && stripos($iri, 'https://') !== 0
        ) {
            continue;
        }

        //Logging
        echo " -- Poll: [$iri].\n";

        //Attempt to mark the feed for immediate polling
        //$result = poll_feed($iri);
    }

    //logging - visual break
    echo "\n";
}

//Exit
echo "Exiting.\n";