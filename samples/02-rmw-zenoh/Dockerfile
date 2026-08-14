FROM ros:jazzy-ros-base

ENV ROS_DOMAIN_ID=42
ENV RMW_IMPLEMENTATION=rmw_zenoh_cpp
ENV ZENOH_SESSION_CONFIG_URI=/ws/configs/zenoh-client.json5
WORKDIR /ws

RUN apt-get update && apt-get install -y --no-install-recommends \
    python3-colcon-common-extensions \
    ros-jazzy-rclcpp \
    ros-jazzy-std-msgs \
    ros-jazzy-rmw-zenoh-cpp \
    && rm -rf /var/lib/apt/lists/*

COPY cpp /ws/cpp
COPY configs /ws/configs
COPY docker /ws/docker

RUN . /opt/ros/jazzy/setup.sh \
    && cd /ws/cpp \
    && colcon build \
    && chmod +x /ws/docker/*.sh

# Default: quick demo (rmw_zenohd + listener + talker in one container).
CMD ["/ws/docker/run-demo.sh"]
