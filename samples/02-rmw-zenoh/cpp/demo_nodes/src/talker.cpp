#include <chrono>
#include <memory>
#include <string>

#include "rclcpp/rclcpp.hpp"
#include "std_msgs/msg/string.hpp"

using namespace std::chrono_literals;

int main(int argc, char * argv[])
{
  rclcpp::init(argc, argv);
  auto node = rclcpp::Node::make_shared("talker");
  auto publisher = node->create_publisher<std_msgs::msg::String>("/demo/chatter", 10);

  size_t count = 0;
  auto timer = node->create_wall_timer(1s, [&]() {
    auto message = std_msgs::msg::String();
    message.data = "Hello " + std::to_string(count++);
    RCLCPP_INFO(node->get_logger(), "Publishing: '%s'", message.data.c_str());
    publisher->publish(message);
  });

  rclcpp::spin(node);
  rclcpp::shutdown();
  return 0;
}
