#!/usr/bin/env Rscript
# Requires installation of R, at a minimum for apt use:
#     $sudo apt install r-base r-base-core r-recommended
# Version 0.1
if (!require("pacman")) install.packages("pacman")
pacman::p_load(
  psych, ggplot2, table1, patchwork,
  data.table, dplyr,tidyverse, anytime, 
  rjson, stringr, loggit, tidygraph
)
if (file.exists("data.csv")) {
  podping_data <- fread(file="data.csv") 
}
if (file.exists("data-unauthorized.csv")) {
  podping_unathorized_data <- fread(file="data-unauthorized.csv")
}
if (file.exists("data-not-podping_firehose.csv")) {
  not_podping_data <- fread(file="data-not-podping_firehose.csv")
}
if (exists("podping_unathorized_data")) {
  # For now not enough to bother analyzing seperetly
  not_podping_data <- rbind(not_podping_data,podping_unathorized_data)
}
count_not_podping_data_unique <- data.table::uniqueN(not_podping_data)
count_podping_data_unique <- data.table::uniqueN(podping_data)

minutes_watching <- 
  (max(podping_data$timestamp_seen)-min(podping_data$timestamp_seen)) / 60
# for a vector image could use: postscript(file="image-timestamp_delay.ps")
time_stamp_delays <- podping_data$timestamp_seen-podping_data$timestamp_post
png(file="stats/image-timestamp_delay.png",
    width=900, height=600)
plot(
  x=podping_data$timestamp_post,
  y=time_stamp_delays,
  main="Scatter plot of watcher delay in seconds"
)
dev.off()
png(file="stats/image-timestamp_delay_hist.png",
    width=900, height=600)
hist(
  time_stamp_delays,
  main="Histogram of watcher delay in seconds"
)
dev.off()

time_stamp_delays <- not_podping_data$timestamp_seen-not_podping_data$timestamp_post
png(file="stats/image-timestamp_delay-non-podping.png",
    width=900, height=600)
plot(
  x=not_podping_data$timestamp_post,
  y=time_stamp_delays,
  main="Scatter plot of watcher delay in seconds - non-podping posts"
)
dev.off()
png(file="stats/image-timestamp_delay_hist-non-podping.png",
    width=900, height=600)
hist(
  time_stamp_delays,
  main="Histogram of watcher delay in seconds - non-podping posts"
)
dev.off()

# Posts per minute #
####################
write_plot_posts_per_min <- function(data_vals, chart_title) {
  data_vals$posix_time_post <- data_vals$timestamp_post %>%
    anytime() %>%
    as.POSIXct()
  # create bins
  by_mins_podpings <- cut.POSIXt(data_vals$posix_time_post,"1 mins")
  podping_data_mins <- split(data_vals$block_num, by_mins_podpings)
  per_min_chart_data <- lapply(podping_data_mins,FUN=length)
  per_min_chart_data_frame <- cbind(
    as.data.frame(anytime(names(per_min_chart_data))),
    as.data.frame(unlist(per_min_chart_data))
  )
  names(per_min_chart_data_frame) <- c("time_bin","frequency")
  png(file=paste0("stats/",chart_title,".png"),
      width=900, height=600)
  plot(
    x=per_min_chart_data_frame$time_bin,
    y=per_min_chart_data_frame$frequency,
    type = "l",
    xlab="Time",
    ylab="Posts / Minute",
    main=paste0(chart_title, "Post Frequency")
  )
  dev.off()
}
# could filter data to specific time frames...
write_plot_posts_per_min(podping_data,"podping_posts_per_minute")
write_plot_posts_per_min(not_podping_data,"Not_podping_posts_per_minute")

# podping_data
######################
# get the URLs from the json objects
# starting descriptive stats with
# https://bookdown.org/wadetroberts/bookdown-demo/descriptive-statistics-and-data-visualization.html

str(podping_data$json)
# json_str = stringr::str_replace_all(podping_data$json[1],"\\\\n",""),
# need to de-prettify the json
podping_data$json  <- podping_data$json %>% 
  stringr::str_replace_all("\\\\n","") %>%
  stringr::str_replace_all("'","")  %>%
  stringr::str_replace_all('\\"\\"','\\"') 

.getUrlFromPostJson <- function(x) {
  rjson::fromJSON(
    x, 
    unexpected.escape = "skip", 
    simplify = TRUE
  )$urls
} 

head(not_podping_data$timestamp_post,2000
     )

podping_data$json_url <- lapply(podping_data$json,.getUrlFromPostJson)
podcastUrls <- unlist(podping_data$json_url)
length(podcastUrls)
length(unique(podcastUrls))
# Display stuff #
#################
.get_pretty_timestamp_diff <- function(
  start_timestamp,
  end_timestamp,
  seconds_decimal=2
){
  .seconds <-
    (end_timestamp-start_timestamp) 
  .years <- as.integer(.seconds / (365.24*24*60*60))
  .days <- as.integer((.seconds / (365.24*24*60*60)-.years)*365.24)
  .days_decimal <-(.seconds / (365.24*24*60*60)-.years)*365.24-.days
  .hours <- as.integer(.days_decimal*24)
  .hours_decimal <- .days_decimal*24 - .hours
  .minutes <- as.integer(.hours_decimal*60)
  .minutes_decimal <- .hours_decimal*60 - .minutes
  .seconds_display <- round(.minutes_decimal*60,seconds_decimal)
  .time_statement_list <- c(
    ifelse(as.integer(.years),
           paste0(.years,
                  ifelse((.years == 1)," year "," years ")
           ),
           NA
    ),
    ifelse(as.integer(.days),
           paste0(.days,
                  ifelse((.days == 1)," day "," days ")
           ),
           NA
    ),
    ifelse(as.integer(.hours),
           paste0(.hours,
                  ifelse((.hours == 1)," hour "," hours ")
           ),
           NA
    ),
    ifelse(as.integer(.minutes),
           paste0(.minutes,
                  ifelse((.minutes == 1)," minute "," minutes ")
           ),
           NA
    ),
    ifelse(as.integer(.seconds_display),
           paste0(.seconds_display,
                  ifelse((.seconds_display == 1)," second "," seconds ")
           ),
           NA
    )
  )
  .time_statement_list <- na.omit(.time_statement_list)
  paste0(
    paste0(
      .time_statement_list[1:(length(.time_statement_list)-1)],
      collapse=""
    ),
    "and ",
    .time_statement_list[length(.time_statement_list)]
  )
}
time_length_display <- .get_pretty_timestamp_diff(
  min(podping_data$timestamp_seen),
  max(podping_data$timestamp_seen)
)

# Summary Statistics to Log #
#############################
loggit::set_logfile("stats/summaryStats.ndjson")
message(
  'Podping hive "custom json" post summary:\n\t',
  "Post count is ",
  count_podping_data_unique, 
  " (", round(count_podping_data_unique/minutes_watching,2),
  " posts/min)\n\t",
  "Total urls posted is ", 
  length(podcastUrls), 
  " of wich ",
  length(unique(podcastUrls)),
  " are unique\n\t",
  "\t(average of ",
  round(length(podcastUrls)/count_podping_data_unique,2),
  " urls/post)\n\t",
  "All 'other' hive post count is ",
  count_not_podping_data_unique,
  " (", round(count_not_podping_data_unique/minutes_watching,2),
  " posts/min)\n\t",
  "Podping portion of all 'custom json' posts on hive is ",
  round(
    100 * count_podping_data_unique / 
      (count_podping_data_unique+count_not_podping_data_unique),
    5
  ),"%", "\n",
  "From ",
  as.character(anytime(min(podping_data$timestamp_seen),asUTC = TRUE)),
  " UTC to ",
  as.character(anytime(max(podping_data$timestamp_seen),asUTC = TRUE)),
  " UTC \n\t Watched for ",
  time_length_display,
  "\n#podping #Stats"
)
