<?php

// Monitor hive-watcher.py output continuously for notifications
// It's expected you would run this script like so:
//    python3 -u ./hive-watcher.py --old=1 --urls_only | php ./podping_watcher.php
//
// 

while(1) {
    $url = trim(readline());
    if(stripos($url, 'http://') === 0 || stripos($url, 'https://') === 0) {
        echo "PODPING: Polling [$url] for new items.\n";
        poll_feed($url);
    }
}