FROM ros:jazzy-ros-base

ENV ROS_DOMAIN_ID=42
WORKDIR /ws

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    gnupg \
    python3-colcon-common-extensions \
    ros-jazzy-rclcpp \
    ros-jazzy-std-msgs \
    ros-jazzy-rmw-cyclonedds-cpp \
    && rm -rf /var/lib/apt/lists/*

# zenohd + zenoh-bridge-ros2dds 1.9.x (Eclipse apt repo).
RUN curl -fsSL https://download.eclipse.org/zenoh/debian-repo/zenoh-public-key \
    | gpg --dearmor --yes -o /etc/apt/keyrings/zenoh-public-key.gpg \
    && echo 'deb [signed-by=/etc/apt/keyrings/zenoh-public-key.gpg] https://download.eclipse.org/zenoh/debian-repo/ /' \
    > /etc/apt/sources.list.d/zenoh.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends zenohd zenoh-bridge-ros2dds \
    && rm -rf /var/lib/apt/lists/*

COPY cpp /ws/cpp
COPY configs /ws/configs
COPY docker/ros2/run-demo.sh /ws/docker/run-demo.sh

RUN . /opt/ros/jazzy/setup.sh \
    && cd /ws/cpp \
    && colcon build \
    && chmod +x /ws/docker/run-demo.sh

CMD ["/ws/docker/run-demo.sh"]
