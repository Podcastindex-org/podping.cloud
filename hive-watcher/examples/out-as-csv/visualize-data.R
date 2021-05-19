#!/usr/bin/env Rscript
# A place holder script for data exploration and visualization
# (that actually does a little visualization)
#
# Requires installation of R, at a minimum for apt use:
#     $sudo apt install r-base r-base-core r-recommended
#
# Version 0.1
podping_data <- read.csv("data.csv")
plot(
        x=podping_data$timestamp_post,
        y=podping_data$timestamp_seen-podping_data$timestamp_post
)
