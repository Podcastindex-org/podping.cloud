#!/usr/bin/env Rscript
# A place holder script for data exploration and visualization
# (that actually does a little visualization)
#
# Requires installation of R, at a minimum for apt use:
#     $sudo apt install r-base r-base-core r-recommended
#
# Version 0.1
podping_data <- read.csv("data.csv")
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

