import { Col, Row, Button } from "antd";
import React, { useEffect } from "react";
// import { ConnectButton } from '../../components/ConnectButton';
import { useConnectionConfig } from "../../contexts/connection";
import { useMarkets } from "../../contexts/market";
// import { Link } from "react-router-dom";

const chef = require("../../img/chef.svg");
// const barbecue = require("../../img/barbecue.svg");
// const friends = require("../../img/friends.svg");
// const solanaLogo = require("../../img/solanaLogo.svg");
// const polygonLogo = require("../../img/polygonDark.svg");
// const keepLogo = require("../../img/keep.svg");
// const hammer = require("../../img/hammer.svg");
// const meat = require("../../img/meat.svg");
// const ocean = require("../../img/ocean.svg");
// const farmer = require("../../img/farmer.svg");
// const stakeHammer = require("../../img/stakeHammer.svg");

export const HomeView = () => {
  const { marketEmitter, midPriceInUSD } = useMarkets();
  const { tokenMap } = useConnectionConfig();

  useEffect(() => {
    const refreshTotal = () => {};

    const dispose = marketEmitter.onMarket(() => {
      refreshTotal();
    });

    refreshTotal();

    return () => {
      dispose();
    };
  }, [marketEmitter, midPriceInUSD, tokenMap]);

  return (
    <Row gutter={[48, 48]} style={{ marginTop: "120px" }}>
      <Col span={8} offset={2}>
        <div>
          <img style={{ maxWidth: "100%" }} src={chef} alt="chef" />
        </div>
        {/* <p>{balance}</p> */}
      </Col>
      <Col
        span={12}
        style={{
          marginBottom: "50px",
          display: "flex",
          flexDirection: "column",
          alignItems: "start",
          textAlign: "left",
        }}
      >
        <h2 className="title" style={{ marginTop: "1em", marginBottom: 0 }}>
          The easiest way to stake your tokens
        </h2>

        <h3 className="subtext" style={{ marginTop: "1em" }}>
          Marinade.finance is a liquid staking protocol built on Solana. Enjoy
          automatic reinvestment of rewards and immediate access to your tokens
          with no lockup period. <br />
          <br /> Don't just 🥩 stake, 👨‍🍳 use Marinade first!
          <br />
          <br />
        </h3>
        <div>
          <a
            href="https://discord.gg/mGqZA5pjRN"
            target="_blank"
            rel="noopener noreferrer"
          >
            {" "}
            <Button size="large" className="tenderButton tenderButtonShade">
              Join our Discord
            </Button>
          </a>
        </div>
      </Col>

      {/* <Col
        span={22}
        offset={1}
        style={{
          marginTop: "20px",
          height: "450",
          display: "flex",
          justifyContent: "space-around",
        }}
      >
        <Card className="card" style={{ width: "30%" }}>
          <div
            style={{
              height: "100%",
              display: "flex",
              flexDirection: "column",
              justifyContent: "space-between",
              alignItems: "center",
            }}
          >
            <div>
              <img
                src={polygonLogo}
                alt="polygon logo"
                style={{ maxWidth: "70%" }}
              />
              <h2 className="subtext" style={{ marginTop: "5px" }}>
                POLYGON
              </h2>
            </div>
            <div style={{ marginTop: "5px", marginBottom: "5px" }}>
              <h2 className="title" style={{ marginBottom: 0 }}>
                10.2%
              </h2>
              <h3 style={{ marginBottom: 0 }}>Rewards</h3>
              <h5>(projected APY)</h5>
            </div>
            <Button className="tenderButton tenderButtonShade" disabled>
              Coming Soon
            </Button>
          </div>
        </Card>
        <Card className="card" style={{ width: "30%", height: "100%" }}>
          <div
            style={{
              height: "100%",
              display: "flex",
              flexDirection: "column",
              justifyContent: "space-between",
              alignItems: "center",
            }}
          >
            <div>
              <img
                src={solanaLogo}
                alt="solana logo"
                style={{ maxWidth: "70%" }}
              />
              <h2 className="subtext" style={{ marginTop: "5px" }}>
                SOLANA
              </h2>
            </div>
            <div style={{ marginTop: "5px", marginBottom: "5px" }}>
              <h2 className="title" style={{ marginBottom: 0 }}>
                15.5%
              </h2>
              <h3 style={{ marginBottom: 0 }}>Rewards</h3>
              <h5>(projected APY)</h5>
            </div>
            {/* <ConnectButton
              className='tenderButton tenderButtonShade'
              onClick={() => handleClick('/discover')}
            >
              Stake
            </ConnectButton>

            <Link to="/discover" style={{ width: "100%" }}>
              <Button className="tenderButton tenderButtonShade">
                Discover
              </Button>
            </Link>
          </div>
        </Card>
        <Card className="card" style={{ width: "30%" }}>
          <div
            style={{
              height: "100%",
              display: "flex",
              flexDirection: "column",
              justifyContent: "space-between",
              alignItems: "center",
            }}
          >
            <div>
              <img
                src={keepLogo}
                alt="keep logo"
                style={{ maxWidth: "100%" }}
              />
              <h2 className="subtext" style={{ marginTop: "5px" }}>
                KEEP
              </h2>
            </div>
            <div style={{ marginTop: "5px", marginBottom: "5px" }}>
              <h2 className="title" style={{ marginBottom: 0 }}>
                9.8%
              </h2>
              <h3 style={{ marginBottom: 0 }}>Rewards</h3>
              <h5>(projected APY)</h5>
            </div>
            <Button className="tenderButton tenderButtonShade" disabled>
              Coming Soon
            </Button>
          </div>
        </Card>
      </Col>
      <Col span={10} offset={2}>
        <div style={{ marginTop: "100px", marginBottom: "70px" }}>
          <img style={{ maxWidth: "100%" }} src={barbecue} alt="barbecue" />
        </div>
        {/* <p>{balance}</p> 
      </Col>
      <Col span={10}>
        <p className="subtext">Marinade is a liquid staking protocol. </p>
        <p className="subtext">
          {" "}
          It allows you to enjoy your farm-fresh, marinated staking returns
          without the wait of an unbonding period.
        </p>
      </Col>
      <Col span={24}>
        <div style={{ marginBottom: "30px" }}>
          <div className="table card">
            <h1 style={{ fontWeight: 600, marginTop: "20px" }}>
              Why Marinade?
            </h1>
            <div className="table">
              <div>
                <span
                  className="subtext table-left"
                  style={{ textAlign: "center" }}
                >
                  <img
                    width="70"
                    src={stakeHammer}
                    alt="stake hammer icon"
                  ></img>
                </span>
                <p className="subtext">Simplify your staking process.</p>
              </div>
              <div>
                <span
                  className="subtext table-left"
                  style={{ textAlign: "center" }}
                >
                  <img width="50" src={farmer} alt="farmer icon" />
                </span>
                <p className="subtext">Maximize your yield.</p>
              </div>
              <div>
                <span
                  className="subtext table-left"
                  style={{ textAlign: "center", width: "20%" }}
                >
                  <img width="50" src={ocean} alt="ocean icon" />
                </span>
                <p className="subtext">
                  Liquify your stakes and get Marinated tokens.
                </p>
              </div>
            </div>
          </div>
        </div>
      </Col>
      <Col span={24}>
        <div
          style={{ height: "65vh", marginBottom: "30px", marginTop: "80px" }}
        >
          <div className="table card">
            <h1 style={{ fontWeight: 600, marginTop: "20px" }}>
              How does it work?
            </h1>
            <div className="table">
              <div>
                <span className="subtext table-left">Step 1</span>
                <p className="subtext">
                  Browse our stake menu, stake and receive freshly minted
                  Marinated tokens that represent your stake and rewards.
                </p>
              </div>
              <div>
                <span className="subtext table-left">Step 2</span>
                <p className="subtext">
                  Sit down, relax and start receiving staking rewards. As your
                  stakes marinades, the value of your Marinated tokens go up.
                </p>
              </div>
              <div>
                <span className="subtext table-left">Step 3</span>
                <p className="subtext">
                  Skip the waiting periods with your newly marinated stake. Go
                  utilize, liquidize, and collateralize, all while you marinade!
                </p>
              </div>
            </div>
          </div>
        </div>
      </Col>
      <Col span={12} offset={2}>
        <div style={{ marginTop: "80px", marginBottom: "100px" }}>
          <img style={{ maxWidth: "100%" }} src={friends} alt="friends" />
        </div>
      </Col>
      <Col span={8}>
        <p className="subtext">
          Friends don’t let friends stake, unless they marinade.
        </p>
      </Col>
      <Col span={24}>
        <div className="builton" />
      </Col> */}
    </Row>
  );
};
