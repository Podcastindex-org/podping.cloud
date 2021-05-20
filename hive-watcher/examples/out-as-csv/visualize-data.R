#!/usr/bin/env Rscript
# Requires installation of R, at a minimum for apt use:
#     $sudo apt install r-base r-base-core r-recommended
# Version 0.1
# starting descriptive stats with
# https://bookdown.org/wadetroberts/bookdown-demo/descriptive-statistics-and-data-visualization.html
if (!require("pacman")) install.packages("pacman")
pacman::p_load(
  psych, ggplot2, table1, patchwork,
  data.table,dplyr,tidyverse, anytime
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
count_not_podping_data_unique <- data.table::uniqueN(not_podping_data)
count_podping_data_unique <- data.table::uniqueN(podping_data)
minutes_watching <- (
  (max(podping_data$timestamp_seen)-min(podping_data$timestamp_seen)) / (60)
)
message(
  "Hive watchers 'custom json' post (posts can contain multiple feed urls) counts: \n\t From ",
  anytime(min(podping_data$timestamp_seen)),
  " to ",
  anytime(max(podping_data$timestamp_seen)),
  "\n\t podping post count = ",
  count_podping_data_unique, 
  " (", round(count_podping_data_unique/minutes_watching,2),
  " posts/min)",
  "\n\t all other post count = ",
  count_not_podping_data_unique,
  " (", round(count_not_podping_data_unique/minutes_watching,2),
  " posts/min)",
  "\n\t podping portion of all posts on hive is %",
  round(
    100 * count_podping_data_unique / 
    (count_podping_data_unique+count_not_podping_data_unique),
    5
  )
)

# for a vector image could use: postscript(file="image-timestamp_delay.ps")
time_stamp_delays <- podping_data$timestamp_seen-podping_data$timestamp_post
png(file="image-timestamp_delay.png",
    width=600, height=600)
plot(
  x=podping_data$timestamp_post,
  y=time_stamp_delays,
  main="Scatter plot of watcher delay in seconds"
)
dev.off()
png(file="image-timestamp_delay_hist.png",
    width=600, height=600)
hist(
  time_stamp_delays,
  main="Histogram of watcher delay in seconds"
)
dev.off()

time_stamp_delays <- not_podping_data$timestamp_seen-not_podping_data$timestamp_post
png(file="image-timestamp_delay-non-podping.png",
    width=600, height=600)
plot(
  x=not_podping_data$timestamp_post,
  y=time_stamp_delays,
  main="Scatter plot of watcher delay in seconds - non-podping posts"
)
dev.off()
png(file="image-timestamp_delay_hist-non-podping.png",
    width=600, height=600)
hist(
  time_stamp_delays,
  main="Histogram of watcher delay in seconds - non-podping posts"
)
dev.off()