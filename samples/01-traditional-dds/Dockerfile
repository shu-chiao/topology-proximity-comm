FROM ros:jazzy-ros-base

ENV ROS_DOMAIN_ID=42
WORKDIR /ws

RUN apt-get update && apt-get install -y --no-install-recommends \
    python3-colcon-common-extensions \
    ros-jazzy-rclcpp \
    ros-jazzy-std-msgs \
    && rm -rf /var/lib/apt/lists/*

COPY cpp /ws/cpp
COPY docker /ws/docker

RUN . /opt/ros/jazzy/setup.sh \
    && cd /ws/cpp \
    && colcon build \
    && chmod +x /ws/docker/*.sh

# Default: quick demo (listener + talker in one container).
CMD ["/ws/docker/run-demo.sh"]
